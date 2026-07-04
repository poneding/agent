use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ids::{ConversationId, TaskId, TurnId};
use crate::timestamp::Timestamp;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlanTodoItemId(String);

impl PlanTodoItemId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    #[must_use]
    pub fn from_string_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for PlanTodoItemId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PlanTodoItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for PlanTodoItemId {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_string_unchecked(value))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoPriority {
    Low,
    Normal,
    High,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoSource {
    Model,
    User,
    Runtime,
    BackgroundTask,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanTodoEvidenceKind {
    Approval,
    Artifact,
    Checkpoint,
    Diagnostic,
    Notification,
    Task,
    Trace,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanTodoEvidence {
    pub kind: PlanTodoEvidenceKind,
    pub reference: String,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanTodoDiagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanTodoItem {
    pub item_id: PlanTodoItemId,
    pub content: String,
    pub status: TodoStatus,
    pub priority: TodoPriority,
    pub source: TodoSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<PlanTodoEvidence>,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanTodoSnapshot {
    pub conversation_id: ConversationId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<TurnId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    pub revision: u64,
    pub items: Vec<PlanTodoItem>,
    pub updated_at: Timestamp,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<PlanTodoDiagnostic>,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanTodoProgress {
    pub total: u32,
    pub pending: u32,
    pub in_progress: u32,
    pub completed: u32,
    pub cancelled: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanTodoSnapshotRecorded {
    pub snapshot: PlanTodoSnapshot,
    pub progress: PlanTodoProgress,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanTodoUpdated {
    pub conversation_id: ConversationId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_id: Option<TurnId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    pub previous_revision: u64,
    pub next_revision: u64,
    pub reason: String,
    pub progress: PlanTodoProgress,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_todo_enums_use_snake_case() {
        assert_eq!(
            serde_json::to_string(&TodoStatus::InProgress).expect("status"),
            "\"in_progress\""
        );
        assert_eq!(
            serde_json::to_string(&TodoPriority::High).expect("priority"),
            "\"high\""
        );
        assert_eq!(
            serde_json::to_string(&TodoSource::BackgroundTask).expect("source"),
            "\"background_task\""
        );
        assert_eq!(
            serde_json::to_string(&PlanTodoEvidenceKind::Checkpoint).expect("evidence"),
            "\"checkpoint\""
        );
    }
}
