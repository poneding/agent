use agent_types::{ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::SchedulerError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    OneShot,
    Background,
    Cron,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskRun {
    pub task_id: TaskId,
    pub workspace_id: WorkspaceId,
    pub conversation_id: Option<ConversationId>,
    pub kind: TaskKind,
    pub state: TaskState,
    pub priority: TaskPriority,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub attempt: u32,
    pub max_attempts: u32,
    pub summary: String,
}

impl TaskRun {
    pub fn new(
        workspace_id: WorkspaceId,
        conversation_id: Option<ConversationId>,
        kind: TaskKind,
        priority: TaskPriority,
        summary: impl Into<String>,
        now: Timestamp,
    ) -> Result<Self, SchedulerError> {
        let summary = summary.into();
        if summary.trim().is_empty() {
            return Err(SchedulerError::InvalidTask(
                "task summary cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            task_id: TaskId::new(),
            workspace_id,
            conversation_id,
            kind,
            state: TaskState::Created,
            priority,
            created_at: now,
            updated_at: now,
            attempt: 0,
            max_attempts: 1,
            summary,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskTransition {
    pub from: TaskState,
    pub to: TaskState,
    pub reason: String,
    pub at: Timestamp,
}

pub fn transition_task_state(
    run: &TaskRun,
    transition: TaskTransition,
) -> Result<TaskRun, SchedulerError> {
    if transition.reason.trim().is_empty() {
        return Err(SchedulerError::InvalidTransition(
            "transition reason cannot be empty".to_string(),
        ));
    }
    if transition.from != run.state {
        return Err(SchedulerError::InvalidTransition(format!(
            "transition starts at {:?}, but task is {:?}",
            transition.from, run.state
        )));
    }
    if run.state.is_terminal() {
        return Err(SchedulerError::InvalidTransition(format!(
            "terminal task state {:?} cannot transition",
            run.state
        )));
    }
    if !is_allowed_transition(transition.from, transition.to) {
        return Err(SchedulerError::InvalidTransition(format!(
            "task cannot transition from {:?} to {:?}",
            transition.from, transition.to
        )));
    }

    let mut next = run.clone();
    next.state = transition.to;
    next.updated_at = transition.at;
    Ok(next)
}

fn is_allowed_transition(from: TaskState, to: TaskState) -> bool {
    match from {
        TaskState::Created => matches!(to, TaskState::Queued),
        TaskState::Queued => matches!(to, TaskState::Running),
        TaskState::Running => matches!(
            to,
            TaskState::WaitingApproval
                | TaskState::Sleeping
                | TaskState::Paused
                | TaskState::Succeeded
                | TaskState::Failed
                | TaskState::Cancelled
        ),
        TaskState::WaitingApproval => matches!(to, TaskState::Running | TaskState::Cancelled),
        TaskState::Sleeping => matches!(
            to,
            TaskState::Queued | TaskState::Cancelled | TaskState::Expired
        ),
        TaskState::Paused => matches!(to, TaskState::Queued | TaskState::Cancelled),
        TaskState::Succeeded | TaskState::Failed | TaskState::Cancelled | TaskState::Expired => {
            false
        }
    }
}
