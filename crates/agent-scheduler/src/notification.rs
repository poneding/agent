use agent_types::{TaskId, TaskNotificationRecorded, Timestamp};
use serde::{Deserialize, Serialize};

use crate::SchedulerError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskNotificationKind {
    Queued,
    Leased,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
    Expired,
    QuotaRejected,
}

impl TaskNotificationKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Leased => "leased",
            Self::WaitingApproval => "waiting_approval",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
            Self::QuotaRejected => "quota_rejected",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskNotification {
    pub task_id: TaskId,
    pub kind: TaskNotificationKind,
    pub summary: String,
    pub occurred_at: Timestamp,
}

impl TaskNotification {
    pub fn new(
        task_id: TaskId,
        kind: TaskNotificationKind,
        summary: impl Into<String>,
        occurred_at: Timestamp,
    ) -> Result<Self, SchedulerError> {
        let summary = summary.into().trim().to_string();
        if summary.is_empty() {
            return Err(SchedulerError::InvalidTask(
                "task notification summary cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            task_id,
            kind,
            summary,
            occurred_at,
        })
    }

    #[must_use]
    pub fn recorded(&self) -> TaskNotificationRecorded {
        TaskNotificationRecorded {
            task_id: self.task_id,
            kind: self.kind.as_str().to_string(),
            summary: self.summary.clone(),
        }
    }
}

pub trait TaskNotificationSink {
    fn notify(&mut self, notification: TaskNotification) -> Result<(), SchedulerError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryTaskNotificationSink {
    notifications: Vec<TaskNotification>,
}

impl InMemoryTaskNotificationSink {
    #[must_use]
    pub fn notifications(&self) -> &[TaskNotification] {
        &self.notifications
    }
}

impl TaskNotificationSink for InMemoryTaskNotificationSink {
    fn notify(&mut self, notification: TaskNotification) -> Result<(), SchedulerError> {
        self.notifications.push(notification);
        Ok(())
    }
}
