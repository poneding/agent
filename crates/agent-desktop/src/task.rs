use agent_types::{
    ApprovalRequestId, ConversationId, PlanTodoItemId, PlanTodoProgress, RecoveryConflictSummary,
    TaskId, TaskState, Timestamp, TodoPriority, WorkspaceId,
};
use agent_ui_bridge::{
    UiTaskActiveTodoSummary, UiTaskCommandError, UiTaskReviewSummary, UiTaskReviewSummaryArtifact,
    UiTaskSnapshot, UiTaskSummary,
};
use serde::{Deserialize, Serialize};

use crate::DesktopError;
use crate::conversation::validate_non_empty;

const MAX_TASK_TEXT_CHARS: usize = 240;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskItem {
    pub task_id: TaskId,
    pub workspace_id: WorkspaceId,
    pub conversation_id: Option<ConversationId>,
    pub state: TaskState,
    pub summary: String,
    pub updated_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_progress: Option<PlanTodoProgress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo: Option<DesktopTaskActiveTodo>,
    #[serde(default)]
    pub waiting_approval: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiting_approval_id: Option<ApprovalRequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_notification: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_notification_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_notification_at: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_notification_approval_id: Option<ApprovalRequestId>,
    #[serde(default)]
    pub artifact_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recovery_conflicts: Vec<RecoveryConflictSummary>,
}

impl DesktopTaskItem {
    pub fn new(
        task_id: TaskId,
        workspace_id: WorkspaceId,
        conversation_id: Option<ConversationId>,
        state: TaskState,
        summary: impl Into<String>,
        updated_at: Timestamp,
    ) -> Result<Self, DesktopError> {
        Ok(Self {
            task_id,
            workspace_id,
            conversation_id,
            state,
            summary: validate_non_empty("task summary", summary.into())?,
            updated_at,
            plan_progress: None,
            active_todo: None,
            waiting_approval: state == TaskState::WaitingApproval,
            waiting_approval_id: None,
            latest_notification: None,
            latest_notification_kind: None,
            latest_notification_at: None,
            latest_notification_approval_id: None,
            artifact_count: 0,
            recovery_conflicts: Vec::new(),
        })
    }

    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("task summary", self.summary.clone())?;
        Ok(())
    }
}

impl TryFrom<&UiTaskSummary> for DesktopTaskItem {
    type Error = DesktopError;

    fn try_from(summary: &UiTaskSummary) -> Result<Self, Self::Error> {
        let (task_summary, _) =
            bounded_required_task_text("task summary", summary.prompt_summary.clone())?;
        let mut item = Self::new(
            summary.task_id,
            summary.workspace_id,
            summary.conversation_id,
            summary.state,
            task_summary,
            summary.updated_at,
        )?;
        item.plan_progress = summary.plan_progress;
        item.active_todo = summary
            .active_todo
            .as_ref()
            .map(DesktopTaskActiveTodo::try_from)
            .transpose()?;
        item.waiting_approval = summary.waiting_approval
            || summary.state == TaskState::WaitingApproval
            || summary.waiting_approval_id.is_some();
        item.waiting_approval_id = summary.waiting_approval_id;
        item.latest_notification = summary
            .latest_notification
            .clone()
            .map(bounded_optional_task_text)
            .transpose()?
            .flatten()
            .map(|item| item.0);
        item.latest_notification_kind = summary
            .latest_notification_kind
            .clone()
            .map(bounded_optional_task_text)
            .transpose()?
            .flatten()
            .map(|item| item.0);
        item.latest_notification_at = summary.latest_notification_at;
        item.latest_notification_approval_id = summary.latest_notification_approval_id;
        item.artifact_count = summary.artifact_count;
        item.recovery_conflicts = summary.recovery_conflicts.clone();
        Ok(item)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskActiveTodo {
    pub item_id: PlanTodoItemId,
    pub content: String,
    pub priority: TodoPriority,
    pub evidence_count: u32,
}

impl DesktopTaskActiveTodo {
    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("active todo content", self.content.clone())?;
        Ok(())
    }
}

impl TryFrom<&UiTaskActiveTodoSummary> for DesktopTaskActiveTodo {
    type Error = DesktopError;

    fn try_from(summary: &UiTaskActiveTodoSummary) -> Result<Self, Self::Error> {
        let (content, _) =
            bounded_required_task_text("active todo content", summary.content.clone())?;
        let item = Self {
            item_id: summary.item_id.clone(),
            content,
            priority: summary.priority,
            evidence_count: summary.evidence_count,
        };
        item.validate()?;
        Ok(item)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskMonitor {
    pub counts: DesktopTaskMonitorCounts,
    pub tasks: Vec<DesktopTaskMonitorItem>,
    pub errors: Vec<DesktopTaskMonitorError>,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskMonitorCounts {
    pub total: u32,
    pub active: u32,
    pub waiting_approval: u32,
    pub failed: u32,
    pub terminal: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskMonitorItem {
    pub task_id: TaskId,
    pub workspace_id: WorkspaceId,
    pub conversation_id: Option<ConversationId>,
    pub state: TaskState,
    pub summary: String,
    pub latest_run_summary: Option<String>,
    pub plan_progress: Option<PlanTodoProgress>,
    pub active_todo: Option<DesktopTaskActiveTodo>,
    pub waiting_approval: bool,
    pub waiting_approval_id: Option<ApprovalRequestId>,
    pub latest_notification: Option<String>,
    pub latest_notification_kind: Option<String>,
    pub latest_notification_at: Option<Timestamp>,
    pub latest_notification_approval_id: Option<ApprovalRequestId>,
    pub artifact_count: u32,
    pub recovery_conflicts: Vec<RecoveryConflictSummary>,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskMonitorError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub truncated: bool,
}

pub fn build_task_monitor(snapshot: &UiTaskSnapshot) -> Result<DesktopTaskMonitor, DesktopError> {
    let mut tasks = snapshot
        .tasks
        .iter()
        .map(DesktopTaskMonitorItem::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    tasks.sort_by(|left, right| {
        left.state
            .is_terminal()
            .cmp(&right.state.is_terminal())
            .then_with(|| right.updated_at.cmp(&left.updated_at))
            .then_with(|| left.task_id.cmp(&right.task_id))
    });
    let errors = snapshot
        .errors
        .iter()
        .map(DesktopTaskMonitorError::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DesktopTaskMonitor {
        counts: build_task_monitor_counts(&tasks),
        tasks,
        errors,
    })
}

impl TryFrom<&UiTaskSummary> for DesktopTaskMonitorItem {
    type Error = DesktopError;

    fn try_from(summary: &UiTaskSummary) -> Result<Self, Self::Error> {
        Ok(Self {
            task_id: summary.task_id,
            workspace_id: summary.workspace_id,
            conversation_id: summary.conversation_id,
            state: summary.state,
            summary: bounded_required_task_text("task summary", summary.prompt_summary.clone())?.0,
            latest_run_summary: summary
                .latest_run_summary
                .clone()
                .map(bounded_optional_task_text)
                .transpose()?
                .flatten()
                .map(|item| item.0),
            plan_progress: summary.plan_progress,
            active_todo: summary
                .active_todo
                .as_ref()
                .map(DesktopTaskActiveTodo::try_from)
                .transpose()?,
            waiting_approval: summary.waiting_approval
                || summary.state == TaskState::WaitingApproval
                || summary.waiting_approval_id.is_some(),
            waiting_approval_id: summary.waiting_approval_id,
            latest_notification: summary
                .latest_notification
                .clone()
                .map(bounded_optional_task_text)
                .transpose()?
                .flatten()
                .map(|item| item.0),
            latest_notification_kind: summary
                .latest_notification_kind
                .clone()
                .map(bounded_optional_task_text)
                .transpose()?
                .flatten()
                .map(|item| item.0),
            latest_notification_at: summary.latest_notification_at,
            latest_notification_approval_id: summary.latest_notification_approval_id,
            artifact_count: summary.artifact_count,
            recovery_conflicts: summary.recovery_conflicts.clone(),
            updated_at: summary.updated_at,
        })
    }
}

impl TryFrom<&UiTaskCommandError> for DesktopTaskMonitorError {
    type Error = DesktopError;

    fn try_from(error: &UiTaskCommandError) -> Result<Self, Self::Error> {
        let (code, code_truncated) =
            bounded_required_task_text("task monitor error code", error.code.clone())?;
        let (message, message_truncated) =
            bounded_required_task_text("task monitor error message", error.message.clone())?;
        Ok(Self {
            code,
            message,
            recoverable: error.recoverable,
            truncated: code_truncated || message_truncated,
        })
    }
}

fn build_task_monitor_counts(tasks: &[DesktopTaskMonitorItem]) -> DesktopTaskMonitorCounts {
    let mut counts = DesktopTaskMonitorCounts {
        total: usize_to_u32(tasks.len()),
        ..DesktopTaskMonitorCounts::default()
    };
    for task in tasks {
        if task.state.is_terminal() {
            counts.terminal = counts.terminal.saturating_add(1);
        }
        match task.state {
            TaskState::WaitingApproval => {
                counts.waiting_approval = counts.waiting_approval.saturating_add(1);
            }
            TaskState::Failed => {
                counts.failed = counts.failed.saturating_add(1);
            }
            state if !state.is_terminal() => {
                counts.active = counts.active.saturating_add(1);
            }
            _ => {}
        }
    }
    counts
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskReviewItem {
    pub task_id: TaskId,
    pub final_state: TaskState,
    pub summary: String,
    pub artifact_count: u32,
    pub trace_id: Option<String>,
    pub artifacts: Vec<DesktopTaskArtifactItem>,
    pub diagnostics: Vec<String>,
}

impl DesktopTaskReviewItem {
    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("task review summary", self.summary.clone())?;
        for artifact in &self.artifacts {
            artifact.validate()?;
        }
        Ok(())
    }
}

impl TryFrom<&UiTaskReviewSummary> for DesktopTaskReviewItem {
    type Error = DesktopError;

    fn try_from(summary: &UiTaskReviewSummary) -> Result<Self, Self::Error> {
        let item = Self {
            task_id: summary.task_id,
            final_state: summary.final_state,
            summary: validate_non_empty("task review summary", summary.summary.clone())?,
            artifact_count: summary.artifact_count,
            trace_id: summary.trace_id.clone(),
            artifacts: summary
                .artifacts
                .iter()
                .map(DesktopTaskArtifactItem::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            diagnostics: summary.diagnostics.clone(),
        };
        item.validate()?;
        Ok(item)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskArtifactItem {
    pub artifact_id: String,
    pub kind: String,
    pub path: Option<String>,
    pub content_hash: Option<String>,
    pub summary: String,
    pub missing: bool,
    pub diagnostics: Vec<String>,
}

impl DesktopTaskArtifactItem {
    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("task artifact id", self.artifact_id.clone())?;
        validate_non_empty("task artifact kind", self.kind.clone())?;
        validate_non_empty("task artifact summary", self.summary.clone())?;
        Ok(())
    }
}

impl TryFrom<&UiTaskReviewSummaryArtifact> for DesktopTaskArtifactItem {
    type Error = DesktopError;

    fn try_from(artifact: &UiTaskReviewSummaryArtifact) -> Result<Self, Self::Error> {
        let item = Self {
            artifact_id: validate_non_empty("task artifact id", artifact.artifact_id.clone())?,
            kind: validate_non_empty("task artifact kind", artifact.kind.clone())?,
            path: artifact.path.clone(),
            content_hash: artifact.content_hash.clone(),
            summary: validate_non_empty("task artifact summary", artifact.summary.clone())?,
            missing: artifact.missing,
            diagnostics: artifact.diagnostics.clone(),
        };
        item.validate()?;
        Ok(item)
    }
}

fn bounded_required_task_text(
    label: &str,
    value: impl Into<String>,
) -> Result<(String, bool), DesktopError> {
    let value = value.into();
    let value = validate_non_empty(label, value)?;
    Ok(bound_task_text(&value))
}

fn bounded_optional_task_text(value: String) -> Result<Option<(String, bool)>, DesktopError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(bound_task_text(value)))
}

fn bound_task_text(value: &str) -> (String, bool) {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() > MAX_TASK_TEXT_CHARS {
        (redacted.chars().take(MAX_TASK_TEXT_CHARS).collect(), true)
    } else {
        (redacted, false)
    }
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            if is_secret_like(token) {
                "[redacted]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
        || value.starts_with("sk-")
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
