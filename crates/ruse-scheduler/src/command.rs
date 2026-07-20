use std::collections::BTreeMap;

use ruse_types::{ApprovalRequestId, ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::{
    SchedulerError, TaskKind, TaskNotificationKind, TaskNotificationSummary, TaskPriority, TaskRun,
    TaskTransition, transition_task_state,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BackgroundAgentTask {
    pub task_id: TaskId,
    pub workspace_id: WorkspaceId,
    pub conversation_id: Option<ConversationId>,
    pub prompt: String,
    pub schedule: TaskSchedule,
    pub state: TaskState,
    pub priority: TaskPriority,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub latest_run_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiting_approval_id: Option<ApprovalRequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_notification: Option<TaskNotificationSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TaskSchedule {
    OneShot { run_at: Option<Timestamp> },
    Cron { expression: String },
}

impl TaskSchedule {
    pub fn validate(&self) -> Result<(), SchedulerError> {
        match self {
            Self::OneShot { .. } => Ok(()),
            Self::Cron { expression } if expression.trim().is_empty() => Err(
                SchedulerError::InvalidTask("cron expression cannot be empty".to_string()),
            ),
            Self::Cron { .. } => Ok(()),
        }
    }

    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::OneShot {
                run_at: Some(run_at),
            } => format!("once at {run_at:?}"),
            Self::OneShot { run_at: None } => "once".to_string(),
            Self::Cron { expression } => format!("cron {expression}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskCreateRequest {
    pub workspace_id: WorkspaceId,
    pub conversation_id: Option<ConversationId>,
    pub prompt: String,
    pub schedule: TaskSchedule,
    pub priority: TaskPriority,
}

impl TaskCreateRequest {
    pub fn new(
        workspace_id: WorkspaceId,
        conversation_id: Option<ConversationId>,
        prompt: impl Into<String>,
        schedule: TaskSchedule,
        priority: TaskPriority,
    ) -> Result<Self, SchedulerError> {
        let prompt = prompt.into().trim().to_string();
        if prompt.is_empty() {
            return Err(SchedulerError::InvalidTask(
                "task prompt cannot be empty".to_string(),
            ));
        }
        schedule.validate()?;

        Ok(Self {
            workspace_id,
            conversation_id,
            prompt,
            schedule,
            priority,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskScheduleUpdate {
    pub task_id: TaskId,
    pub schedule: TaskSchedule,
    pub reason: String,
}

impl TaskScheduleUpdate {
    pub fn new(
        task_id: TaskId,
        schedule: TaskSchedule,
        reason: impl Into<String>,
    ) -> Result<Self, SchedulerError> {
        schedule.validate()?;
        let reason = normalize_reason(reason)?;
        Ok(Self {
            task_id,
            schedule,
            reason,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum TaskCommand {
    Create(TaskCreateRequest),
    List,
    Inspect { task_id: TaskId },
    Pause { task_id: TaskId, reason: String },
    Resume { task_id: TaskId, reason: String },
    Cancel { task_id: TaskId, reason: String },
    UpdateSchedule(TaskScheduleUpdate),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum TaskCommandResponse {
    Created(BackgroundAgentTask),
    Listed(Vec<BackgroundAgentTask>),
    Inspected(BackgroundAgentTask),
    Paused(BackgroundAgentTask),
    Resumed(BackgroundAgentTask),
    Cancelled(BackgroundAgentTask),
    ScheduleUpdated(BackgroundAgentTask),
}

pub trait TaskCommandService {
    fn handle(
        &mut self,
        command: TaskCommand,
        now: Timestamp,
    ) -> Result<TaskCommandResponse, SchedulerError>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryTaskCommandService {
    tasks: BTreeMap<TaskId, BackgroundAgentTask>,
}

impl InMemoryTaskCommandService {
    pub fn handle(
        &mut self,
        command: TaskCommand,
        now: Timestamp,
    ) -> Result<TaskCommandResponse, SchedulerError> {
        TaskCommandService::handle(self, command, now)
    }

    #[must_use]
    pub fn list(&self) -> Vec<BackgroundAgentTask> {
        let mut tasks = self.tasks.values().cloned().collect::<Vec<_>>();
        tasks.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then_with(|| left.task_id.cmp(&right.task_id))
        });
        tasks
    }

    #[must_use]
    pub fn get(&self, task_id: TaskId) -> Option<&BackgroundAgentTask> {
        self.tasks.get(&task_id)
    }

    fn create(
        &mut self,
        request: TaskCreateRequest,
        now: Timestamp,
    ) -> Result<BackgroundAgentTask, SchedulerError> {
        let created = BackgroundAgentTask {
            task_id: TaskId::new(),
            workspace_id: request.workspace_id,
            conversation_id: request.conversation_id,
            prompt: request.prompt,
            schedule: request.schedule,
            state: TaskState::Created,
            priority: request.priority,
            created_at: now,
            updated_at: now,
            latest_run_summary: None,
            waiting_approval_id: None,
            latest_notification: None,
        };
        let mut queued = transition_task(&created, TaskState::Queued, "enqueue", now)?;
        record_task_notification(
            &mut queued,
            TaskNotificationKind::Queued,
            "task queued",
            now,
            None,
        )?;
        self.tasks.insert(queued.task_id, queued.clone());
        Ok(queued)
    }

    fn inspect(&self, task_id: TaskId) -> Result<BackgroundAgentTask, SchedulerError> {
        self.tasks
            .get(&task_id)
            .cloned()
            .ok_or_else(|| SchedulerError::InvalidTask(format!("task not found: {task_id}")))
    }

    fn update_task_state(
        &mut self,
        task_id: TaskId,
        to: TaskState,
        notification_kind: TaskNotificationKind,
        reason: impl Into<String>,
        now: Timestamp,
    ) -> Result<BackgroundAgentTask, SchedulerError> {
        let task = self.inspect(task_id)?;
        let reason = normalize_reason(reason)?;
        let mut next = transition_task(&task, to, reason.clone(), now)?;
        if matches!(
            to,
            TaskState::Queued | TaskState::Cancelled | TaskState::Paused
        ) {
            next.waiting_approval_id = None;
        }
        record_task_notification(&mut next, notification_kind, reason, now, None)?;
        self.tasks.insert(task_id, next.clone());
        Ok(next)
    }

    fn update_schedule(
        &mut self,
        update: TaskScheduleUpdate,
        now: Timestamp,
    ) -> Result<BackgroundAgentTask, SchedulerError> {
        let mut task = self.inspect(update.task_id)?;
        if task.state.is_terminal() {
            return Err(SchedulerError::InvalidTransition(format!(
                "terminal task state {:?} cannot update schedule",
                task.state
            )));
        }
        task.schedule = update.schedule;
        task.updated_at = now;
        record_task_notification(
            &mut task,
            TaskNotificationKind::ScheduleUpdated,
            update.reason,
            now,
            None,
        )?;
        self.tasks.insert(task.task_id, task.clone());
        Ok(task)
    }
}

impl TaskCommandService for InMemoryTaskCommandService {
    fn handle(
        &mut self,
        command: TaskCommand,
        now: Timestamp,
    ) -> Result<TaskCommandResponse, SchedulerError> {
        match command {
            TaskCommand::Create(request) => {
                self.create(request, now).map(TaskCommandResponse::Created)
            }
            TaskCommand::List => Ok(TaskCommandResponse::Listed(self.list())),
            TaskCommand::Inspect { task_id } => {
                self.inspect(task_id).map(TaskCommandResponse::Inspected)
            }
            TaskCommand::Pause { task_id, reason } => self
                .update_task_state(
                    task_id,
                    TaskState::Paused,
                    TaskNotificationKind::Paused,
                    reason,
                    now,
                )
                .map(TaskCommandResponse::Paused),
            TaskCommand::Resume { task_id, reason } => self
                .update_task_state(
                    task_id,
                    TaskState::Queued,
                    TaskNotificationKind::Resumed,
                    reason,
                    now,
                )
                .map(TaskCommandResponse::Resumed),
            TaskCommand::Cancel { task_id, reason } => self
                .update_task_state(
                    task_id,
                    TaskState::Cancelled,
                    TaskNotificationKind::Cancelled,
                    reason,
                    now,
                )
                .map(TaskCommandResponse::Cancelled),
            TaskCommand::UpdateSchedule(update) => self
                .update_schedule(update, now)
                .map(TaskCommandResponse::ScheduleUpdated),
        }
    }
}

fn transition_task(
    task: &BackgroundAgentTask,
    to: TaskState,
    reason: impl Into<String>,
    at: Timestamp,
) -> Result<BackgroundAgentTask, SchedulerError> {
    let run = TaskRun {
        task_id: task.task_id,
        workspace_id: task.workspace_id,
        conversation_id: task.conversation_id,
        kind: TaskKind::Background,
        state: task.state,
        priority: task.priority,
        created_at: task.created_at,
        updated_at: task.updated_at,
        attempt: 0,
        max_attempts: 1,
        summary: task_summary(task),
        waiting_approval_id: task.waiting_approval_id,
        latest_notification: task.latest_notification.clone(),
    };
    let transitioned = transition_task_state(
        &run,
        TaskTransition {
            from: task.state,
            to,
            reason: reason.into(),
            at,
        },
    )?;
    let mut next = task.clone();
    next.state = transitioned.state;
    next.updated_at = transitioned.updated_at;
    Ok(next)
}

fn record_task_notification(
    task: &mut BackgroundAgentTask,
    kind: TaskNotificationKind,
    summary: impl Into<String>,
    occurred_at: Timestamp,
    approval_id: Option<ApprovalRequestId>,
) -> Result<(), SchedulerError> {
    task.latest_notification = Some(TaskNotificationSummary::new(
        kind,
        summary,
        occurred_at,
        approval_id,
    )?);
    Ok(())
}

fn task_summary(task: &BackgroundAgentTask) -> String {
    let redacted = redact_task_text(&task.prompt);
    if redacted.chars().count() > 80 {
        redacted.chars().take(80).collect()
    } else {
        redacted
    }
}

fn normalize_reason(reason: impl Into<String>) -> Result<String, SchedulerError> {
    let reason = reason.into();
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(SchedulerError::InvalidTransition(
            "task command reason cannot be empty".to_string(),
        ));
    }
    Ok(redact_task_text(reason))
}

#[must_use]
pub fn redact_task_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            if is_secret_like_token(part) {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like_token(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}
