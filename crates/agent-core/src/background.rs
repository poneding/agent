use agent_model::{ModelProviderRegistry, ModelRequestRequirements};
use agent_scheduler::{RetryDecision, SchedulerError, TaskQueue, TaskRun, redact_task_text};
use agent_types::{CorrelationId, TaskId, TaskState, Timestamp, TurnId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AgentLoopBudget, AgentLoopTerminalState, ComprehensiveTurnInput, ComprehensiveTurnRunner,
    EffectiveRuntimeConfig, RuntimeIds,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackgroundTaskRunnerInput {
    pub worker_id: String,
    pub now: Timestamp,
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
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackgroundTaskRunOutcome {
    pub task_id: TaskId,
    pub final_state: TaskState,
    pub event_count: usize,
    pub trace_id: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackgroundTaskTurnResult {
    pub event_count: usize,
    pub trace_id: Option<String>,
    pub summary: String,
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
        let Some(lease) = queue.acquire_next(&input.worker_id, input.now)? else {
            return Ok(None);
        };

        let task = queue
            .get_task(lease.task_id)
            .cloned()
            .ok_or(BackgroundRunnerError::MissingLeasedTask(lease.task_id))?;

        match executor.execute_task_turn(&task).await {
            Ok(result) => {
                let completed = queue.complete(&lease.id, input.now)?;
                Ok(Some(BackgroundTaskRunOutcome {
                    task_id: completed.task_id,
                    final_state: completed.state,
                    event_count: result.event_count,
                    trace_id: normalize_trace_id(result.trace_id),
                    summary: bounded_summary(&result.summary),
                }))
            }
            Err(error) => {
                let decision = queue.fail(&lease.id, error.message.clone(), input.now)?;
                Ok(Some(outcome_from_retry_decision(
                    decision,
                    error.message,
                    None,
                )))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ComprehensiveBackgroundTurnExecutor {
    effective_config: EffectiveRuntimeConfig,
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
            effective_config,
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
            effective_config: self.effective_config.clone(),
            requirements: self.requirements.clone(),
            loop_budget: self.loop_budget.clone(),
        };
        let outcome = ComprehensiveTurnRunner::run(input, &self.registry)
            .await
            .map_err(|error| BackgroundTaskTurnError::new(error.to_string()))?;

        match outcome.terminal_state {
            AgentLoopTerminalState::Done => Ok(BackgroundTaskTurnResult {
                event_count: outcome.events.len(),
                trace_id: Some(task.task_id.to_string()),
                summary: outcome
                    .final_message
                    .unwrap_or_else(|| "background task completed".to_string()),
            }),
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
            BackgroundTaskRunOutcome {
                task_id: task.task_id,
                final_state: task.state,
                event_count: 0,
                trace_id,
                summary: bounded_summary(&summary),
            }
        }
    }
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
