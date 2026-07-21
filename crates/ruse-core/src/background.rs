use async_trait::async_trait;
use ruse_model::{ModelProviderRegistry, ModelRequestRequirements};
use ruse_scheduler::{
    RetryDecision, SchedulerError, TaskNotification, TaskNotificationKind, TaskNotificationSink,
    TaskQueue, TaskRun, redact_task_text,
};
use ruse_types::{
    ApprovalRequestId, CheckpointId, CorrelationId, RuntimeEvent, RuntimeEventPayload, TaskId,
    TaskState, Timestamp, TurnId,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AgentLoopBudget, AgentLoopTerminalState, ComprehensiveTurnInput, ComprehensiveTurnRunner,
    EffectiveRuntimeConfig, RuntimeIds, TaskArtifact, TaskReviewSummary,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackgroundTaskRunnerInput {
    pub worker_id: String,
    pub now: Timestamp,
    pub cancellation_requested: bool,
}

impl BackgroundTaskRunnerInput {
    pub fn new(
        worker_id: impl Into<String>,
        now: Timestamp,
    ) -> Result<Self, BackgroundRunnerError> {
        let worker_id = worker_id.into();
        let worker_id = worker_id.trim();
        if worker_id.is_empty() {
            return Err(BackgroundRunnerError::InvalidInput(
                "worker_id cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            worker_id: worker_id.to_string(),
            now,
            cancellation_requested: false,
        })
    }

    #[must_use]
    pub const fn with_cancellation_requested(mut self, cancellation_requested: bool) -> Self {
        self.cancellation_requested = cancellation_requested;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackgroundTaskRunOutcome {
    pub task_id: TaskId,
    pub final_state: TaskState,
    pub event_count: usize,
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<CheckpointId>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<TaskArtifact>,
    pub review: TaskReviewSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<ApprovalRequestId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackgroundTaskTurnResult {
    pub event_count: usize,
    pub trace_id: Option<String>,
    pub checkpoint_id: Option<CheckpointId>,
    pub summary: String,
    pub artifacts: Vec<TaskArtifact>,
    pub approval_id: Option<ApprovalRequestId>,
}

impl BackgroundTaskTurnResult {
    #[must_use]
    pub fn completed(
        summary: impl Into<String>,
        event_count: usize,
        trace_id: Option<String>,
    ) -> Self {
        Self {
            event_count,
            trace_id,
            checkpoint_id: None,
            summary: summary.into(),
            artifacts: Vec::new(),
            approval_id: None,
        }
    }

    #[must_use]
    pub fn waiting_approval(
        approval_id: ApprovalRequestId,
        summary: impl Into<String>,
        event_count: usize,
    ) -> Self {
        Self {
            event_count,
            trace_id: None,
            checkpoint_id: None,
            summary: summary.into(),
            artifacts: Vec::new(),
            approval_id: Some(approval_id),
        }
    }

    #[must_use]
    pub fn with_checkpoint_id(mut self, checkpoint_id: CheckpointId) -> Self {
        self.checkpoint_id = Some(checkpoint_id);
        self
    }

    #[must_use]
    pub fn with_artifacts(mut self, artifacts: Vec<TaskArtifact>) -> Self {
        self.artifacts = artifacts;
        self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BackgroundApprovalDecisionKind {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackgroundApprovalDecision {
    pub task_id: TaskId,
    pub approval_id: ApprovalRequestId,
    pub kind: BackgroundApprovalDecisionKind,
    pub reason: String,
    pub decided_at: Timestamp,
}

impl BackgroundApprovalDecision {
    pub fn new(
        task_id: TaskId,
        approval_id: ApprovalRequestId,
        kind: BackgroundApprovalDecisionKind,
        reason: impl Into<String>,
        decided_at: Timestamp,
    ) -> Result<Self, BackgroundRunnerError> {
        let reason = bounded_summary(&reason.into());
        if reason.is_empty() {
            return Err(BackgroundRunnerError::InvalidInput(
                "approval decision reason cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            task_id,
            approval_id,
            kind,
            reason,
            decided_at,
        })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("background task turn failed: {message}")]
pub struct BackgroundTaskTurnError {
    pub message: String,
}

impl BackgroundTaskTurnError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: bounded_summary(&message.into()),
        }
    }
}

#[async_trait]
pub trait BackgroundTaskTurnExecutor {
    async fn execute_task_turn(
        &mut self,
        task: &TaskRun,
    ) -> Result<BackgroundTaskTurnResult, BackgroundTaskTurnError>;
}

#[derive(Clone, Debug, Default)]
pub struct BackgroundTaskRunner;

impl BackgroundTaskRunner {
    pub async fn run_one<Q, E>(
        &self,
        queue: &mut Q,
        executor: &mut E,
        input: BackgroundTaskRunnerInput,
    ) -> Result<Option<BackgroundTaskRunOutcome>, BackgroundRunnerError>
    where
        Q: TaskQueue + ?Sized,
        E: BackgroundTaskTurnExecutor + Send + ?Sized,
    {
        self.run_one_internal(queue, executor, input, None).await
    }

    pub async fn run_one_with_notifications<Q, E, S>(
        &self,
        queue: &mut Q,
        executor: &mut E,
        notifications: &mut S,
        input: BackgroundTaskRunnerInput,
    ) -> Result<Option<BackgroundTaskRunOutcome>, BackgroundRunnerError>
    where
        Q: TaskQueue + ?Sized,
        E: BackgroundTaskTurnExecutor + Send + ?Sized,
        S: TaskNotificationSink,
    {
        self.run_one_internal(queue, executor, input, Some(notifications))
            .await
    }

    async fn run_one_internal<Q, E>(
        &self,
        queue: &mut Q,
        executor: &mut E,
        input: BackgroundTaskRunnerInput,
        mut notifications: Option<&mut dyn TaskNotificationSink>,
    ) -> Result<Option<BackgroundTaskRunOutcome>, BackgroundRunnerError>
    where
        Q: TaskQueue + ?Sized,
        E: BackgroundTaskTurnExecutor + Send + ?Sized,
    {
        let Some(lease) = queue.acquire_next(&input.worker_id, input.now)? else {
            return Ok(None);
        };

        let task = queue
            .get_task(lease.task_id)
            .cloned()
            .ok_or(BackgroundRunnerError::MissingLeasedTask(lease.task_id))?;

        if input.cancellation_requested {
            let summary = "background task cancellation requested";
            let cancelled = queue.cancel(task.task_id, summary.to_string(), input.now)?;
            let artifacts = Vec::new();
            let review = build_task_review(
                cancelled.task_id,
                cancelled.state,
                None,
                None,
                &artifacts,
                summary,
            )?;
            notify_task(
                &mut notifications,
                cancelled.task_id,
                TaskNotificationKind::Cancelled,
                summary,
                input.now,
                None,
            )?;
            return Ok(Some(BackgroundTaskRunOutcome {
                task_id: cancelled.task_id,
                final_state: cancelled.state,
                event_count: 0,
                trace_id: None,
                checkpoint_id: None,
                summary: summary.to_string(),
                artifacts,
                review,
                approval_id: None,
            }));
        }

        match executor.execute_task_turn(&task).await {
            Ok(result) => {
                let summary = bounded_summary(&result.summary);
                let trace_id = normalize_trace_id(result.trace_id);
                let checkpoint_id = result.checkpoint_id;
                let artifacts = result.artifacts;
                if let Some(approval_id) = result.approval_id {
                    let waiting = queue.wait_for_approval(
                        &lease.id,
                        approval_id,
                        summary.clone(),
                        input.now,
                    )?;
                    notify_task(
                        &mut notifications,
                        waiting.task_id,
                        TaskNotificationKind::WaitingApproval,
                        &summary,
                        input.now,
                        Some(approval_id),
                    )?;
                    let review = build_task_review(
                        waiting.task_id,
                        waiting.state,
                        trace_id.as_deref(),
                        checkpoint_id,
                        &artifacts,
                        &summary,
                    )?;
                    return Ok(Some(BackgroundTaskRunOutcome {
                        task_id: waiting.task_id,
                        final_state: waiting.state,
                        event_count: result.event_count,
                        trace_id,
                        checkpoint_id,
                        summary,
                        artifacts,
                        review,
                        approval_id: Some(approval_id),
                    }));
                }

                let completed = queue.complete(&lease.id, input.now)?;
                notify_task(
                    &mut notifications,
                    completed.task_id,
                    TaskNotificationKind::Completed,
                    &summary,
                    input.now,
                    None,
                )?;
                let review = build_task_review(
                    completed.task_id,
                    completed.state,
                    trace_id.as_deref(),
                    checkpoint_id,
                    &artifacts,
                    &summary,
                )?;
                Ok(Some(BackgroundTaskRunOutcome {
                    task_id: completed.task_id,
                    final_state: completed.state,
                    event_count: result.event_count,
                    trace_id,
                    checkpoint_id,
                    summary,
                    artifacts,
                    review,
                    approval_id: None,
                }))
            }
            Err(error) => {
                let decision = queue.fail(&lease.id, error.message.clone(), input.now)?;
                let outcome = outcome_from_retry_decision(decision, error.message, None);
                if let Some(kind) = notification_kind_for_state(outcome.final_state) {
                    notify_task(
                        &mut notifications,
                        outcome.task_id,
                        kind,
                        &outcome.summary,
                        input.now,
                        None,
                    )?;
                }
                Ok(Some(outcome))
            }
        }
    }

    pub fn resolve_approval<Q, S>(
        &self,
        queue: &mut Q,
        notifications: &mut S,
        decision: BackgroundApprovalDecision,
    ) -> Result<BackgroundTaskRunOutcome, BackgroundRunnerError>
    where
        Q: TaskQueue + ?Sized,
        S: TaskNotificationSink,
    {
        let current_state = queue
            .get_task(decision.task_id)
            .map(|task| task.state)
            .ok_or(BackgroundRunnerError::MissingLeasedTask(decision.task_id))?;
        if current_state != TaskState::WaitingApproval {
            return Err(BackgroundRunnerError::Scheduler(
                SchedulerError::InvalidTransition(format!(
                    "approval can only resolve waiting task, got {current_state:?}"
                )),
            ));
        }

        let (task, notification_kind) = match decision.kind {
            BackgroundApprovalDecisionKind::Allow => (
                queue.resume(
                    decision.task_id,
                    decision.reason.clone(),
                    decision.decided_at,
                )?,
                TaskNotificationKind::Resumed,
            ),
            BackgroundApprovalDecisionKind::Deny => (
                queue.cancel(
                    decision.task_id,
                    decision.reason.clone(),
                    decision.decided_at,
                )?,
                TaskNotificationKind::Cancelled,
            ),
        };
        let summary = bounded_summary(&decision.reason);
        let mut optional_sink: Option<&mut dyn TaskNotificationSink> = Some(notifications);
        notify_task(
            &mut optional_sink,
            task.task_id,
            notification_kind,
            &summary,
            decision.decided_at,
            Some(decision.approval_id),
        )?;
        let artifacts = Vec::new();
        let review = build_task_review(task.task_id, task.state, None, None, &artifacts, &summary)?;

        Ok(BackgroundTaskRunOutcome {
            task_id: task.task_id,
            final_state: task.state,
            event_count: 0,
            trace_id: None,
            checkpoint_id: None,
            summary,
            artifacts,
            review,
            approval_id: Some(decision.approval_id),
        })
    }
}

#[derive(Clone, Debug)]
pub struct ComprehensiveBackgroundTurnExecutor {
    effective_config: std::sync::Arc<EffectiveRuntimeConfig>,
    requirements: ModelRequestRequirements,
    loop_budget: AgentLoopBudget,
    registry: ModelProviderRegistry,
}

impl ComprehensiveBackgroundTurnExecutor {
    #[must_use]
    pub fn new(
        effective_config: EffectiveRuntimeConfig,
        requirements: ModelRequestRequirements,
        loop_budget: AgentLoopBudget,
        registry: ModelProviderRegistry,
    ) -> Self {
        Self {
            effective_config: std::sync::Arc::new(effective_config),
            requirements,
            loop_budget,
            registry,
        }
    }
}

#[async_trait]
impl BackgroundTaskTurnExecutor for ComprehensiveBackgroundTurnExecutor {
    async fn execute_task_turn(
        &mut self,
        task: &TaskRun,
    ) -> Result<BackgroundTaskTurnResult, BackgroundTaskTurnError> {
        let input = ComprehensiveTurnInput {
            ids: RuntimeIds {
                workspace_id: task.workspace_id,
                conversation_id: task.conversation_id.unwrap_or_default(),
                turn_id: TurnId::new(),
                correlation_id: CorrelationId::new(),
            },
            user_message: task.summary.clone(),
            conversation_history: std::sync::Arc::new(Vec::new()),
            effective_config: std::sync::Arc::clone(&self.effective_config),
            requirements: self.requirements.clone(),
            loop_budget: self.loop_budget.clone(),
        };
        let outcome = ComprehensiveTurnRunner::run(input, &self.registry)
            .await
            .map_err(|error| BackgroundTaskTurnError::new(error.to_string()))?;

        match outcome.terminal_state {
            AgentLoopTerminalState::Done => Ok(BackgroundTaskTurnResult::completed(
                outcome
                    .final_message
                    .unwrap_or_else(|| "background task completed".to_string()),
                outcome.events.len(),
                Some(task.task_id.to_string()),
            )),
            AgentLoopTerminalState::WaitingApproval => {
                let approval_id = latest_approval_request_id(&outcome.events).ok_or_else(|| {
                    BackgroundTaskTurnError::new(
                        "background turn is waiting for approval without an approval request",
                    )
                })?;
                Ok(BackgroundTaskTurnResult::waiting_approval(
                    approval_id,
                    "background task waiting for approval",
                    outcome.events.len(),
                ))
            }
            state => Err(BackgroundTaskTurnError::new(format!(
                "background turn ended in {state:?}"
            ))),
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BackgroundRunnerError {
    #[error("invalid background runner input: {0}")]
    InvalidInput(String),
    #[error("leased task is missing: {0}")]
    MissingLeasedTask(TaskId),
    #[error("scheduler failed: {0}")]
    Scheduler(#[from] SchedulerError),
}

fn outcome_from_retry_decision(
    decision: RetryDecision,
    summary: String,
    trace_id: Option<String>,
) -> BackgroundTaskRunOutcome {
    match decision {
        RetryDecision::RetryAt { task, .. } | RetryDecision::Exhausted { task, .. } => {
            let artifacts = Vec::new();
            let summary = bounded_summary(&summary);
            let review = build_task_review(
                task.task_id,
                task.state,
                trace_id.as_deref(),
                None,
                &artifacts,
                &summary,
            )
            .expect("bounded failure summaries are valid task reviews");
            BackgroundTaskRunOutcome {
                task_id: task.task_id,
                final_state: task.state,
                event_count: 0,
                trace_id,
                checkpoint_id: None,
                summary,
                artifacts,
                review,
                approval_id: None,
            }
        }
    }
}

fn build_task_review(
    task_id: TaskId,
    final_state: TaskState,
    trace_id: Option<&str>,
    checkpoint_id: Option<CheckpointId>,
    artifacts: &[TaskArtifact],
    summary: &str,
) -> Result<TaskReviewSummary, BackgroundRunnerError> {
    TaskReviewSummary::from_artifacts(
        task_id,
        final_state,
        trace_id,
        checkpoint_id,
        artifacts,
        summary,
    )
    .map_err(|error| BackgroundRunnerError::InvalidInput(error.to_string()))
}

fn notify_task(
    notifications: &mut Option<&mut dyn TaskNotificationSink>,
    task_id: TaskId,
    kind: TaskNotificationKind,
    summary: &str,
    occurred_at: Timestamp,
    approval_id: Option<ApprovalRequestId>,
) -> Result<(), SchedulerError> {
    let Some(sink) = notifications.as_mut() else {
        return Ok(());
    };
    let mut notification = TaskNotification::new(task_id, kind, summary, occurred_at)?;
    if let Some(approval_id) = approval_id {
        notification = notification.with_approval_id(approval_id);
    }
    sink.notify(notification)
}

fn notification_kind_for_state(state: TaskState) -> Option<TaskNotificationKind> {
    match state {
        TaskState::Queued => Some(TaskNotificationKind::Queued),
        TaskState::Running => Some(TaskNotificationKind::Leased),
        TaskState::WaitingApproval => Some(TaskNotificationKind::WaitingApproval),
        TaskState::Succeeded => Some(TaskNotificationKind::Completed),
        TaskState::Failed => Some(TaskNotificationKind::Failed),
        TaskState::Cancelled => Some(TaskNotificationKind::Cancelled),
        TaskState::Expired => Some(TaskNotificationKind::Expired),
        TaskState::Created | TaskState::Sleeping | TaskState::Paused => None,
    }
}

fn latest_approval_request_id(events: &[RuntimeEvent]) -> Option<ApprovalRequestId> {
    events.iter().rev().find_map(|event| {
        if let RuntimeEventPayload::ApprovalRequested(requested) = &event.payload {
            Some(requested.request_id)
        } else {
            None
        }
    })
}

fn normalize_trace_id(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(bounded_summary(value))
        }
    })
}

fn bounded_summary(value: &str) -> String {
    let redacted = redact_task_text(value);
    let redacted = redacted.trim();
    if redacted.chars().count() > 160 {
        redacted.chars().take(160).collect()
    } else {
        redacted.to_string()
    }
}
