use agent_types::{ApprovalRequestId, TaskId, TaskNotificationRecorded, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{SchedulerError, redact_task_text};

const MAX_NOTIFICATION_SUMMARY_CHARS: usize = 160;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskNotificationKind {
    Queued,
    Leased,
    WaitingApproval,
    Resumed,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Expired,
    QuotaRejected,
    ScheduleUpdated,
}

impl TaskNotificationKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Leased => "leased",
            Self::WaitingApproval => "waiting_approval",
            Self::Resumed => "resumed",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
            Self::QuotaRejected => "quota_rejected",
            Self::ScheduleUpdated => "schedule_updated",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskNotificationSummary {
    pub kind: TaskNotificationKind,
    pub summary: String,
    pub occurred_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<ApprovalRequestId>,
}

impl TaskNotificationSummary {
    pub fn new(
        kind: TaskNotificationKind,
        summary: impl Into<String>,
        occurred_at: Timestamp,
        approval_id: Option<ApprovalRequestId>,
    ) -> Result<Self, SchedulerError> {
        let summary = normalize_notification_summary(summary);
        if summary.is_empty() {
            return Err(SchedulerError::InvalidTask(
                "task notification summary cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            kind,
            summary,
            occurred_at,
            approval_id,
        })
    }

    #[must_use]
    pub fn from_notification(notification: &TaskNotification) -> Self {
        Self {
            kind: notification.kind,
            summary: notification.summary.clone(),
            occurred_at: notification.occurred_at,
            approval_id: notification.approval_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskNotification {
    pub task_id: TaskId,
    pub kind: TaskNotificationKind,
    pub summary: String,
    pub occurred_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_id: Option<ApprovalRequestId>,
}

impl TaskNotification {
    pub fn new(
        task_id: TaskId,
        kind: TaskNotificationKind,
        summary: impl Into<String>,
        occurred_at: Timestamp,
    ) -> Result<Self, SchedulerError> {
        let summary = normalize_notification_summary(summary);
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
            approval_id: None,
        })
    }

    #[must_use]
    pub fn with_approval_id(mut self, approval_id: ApprovalRequestId) -> Self {
        self.approval_id = Some(approval_id);
        self
    }

    #[must_use]
    pub fn recorded(&self) -> TaskNotificationRecorded {
        TaskNotificationRecorded {
            task_id: self.task_id,
            kind: self.kind.as_str().to_string(),
            summary: self.summary.clone(),
            approval_id: self.approval_id,
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
        if self.notifications.iter().any(|existing| {
            existing.task_id == notification.task_id
                && existing.kind == notification.kind
                && existing.approval_id == notification.approval_id
        }) {
            return Ok(());
        }
        self.notifications.push(notification);
        Ok(())
    }
}

fn normalize_notification_summary(summary: impl Into<String>) -> String {
    let redacted = redact_task_text(&summary.into());
    let trimmed = redacted.trim();
    if trimmed.chars().count() > MAX_NOTIFICATION_SUMMARY_CHARS {
        trimmed
            .chars()
            .take(MAX_NOTIFICATION_SUMMARY_CHARS)
            .collect()
    } else {
        trimmed.to_string()
    }
}
