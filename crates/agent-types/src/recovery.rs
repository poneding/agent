use serde::{Deserialize, Serialize};

use crate::{CheckpointId, ConversationId, EventSeq, Timestamp, TurnId};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoverySnapshot {
    pub conversation_id: ConversationId,
    pub turn_id: Option<TurnId>,
    pub checkpoint: Option<RecoveryCheckpointSummary>,
    pub replay: RecoveryReplaySummary,
    pub readiness: RecoveryReadiness,
    pub pending_tool_calls: Vec<RecoveredToolCall>,
    pub idempotent_effects: Vec<RecoveredToolEffect>,
    pub pending_approvals: Vec<RecoveredApprovalWait>,
    pub diagnostics: Vec<RecoveryDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveryCheckpointSummary {
    pub checkpoint_id: CheckpointId,
    pub event_seq: EventSeq,
    pub label: String,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveryReplaySummary {
    pub replay_from: EventSeq,
    pub replayed_event_count: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryReadiness {
    Ready,
    NeedsApproval,
    PendingToolCalls,
    CorruptCheckpoint,
    StorageError,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveredToolCall {
    pub call_id: String,
    pub tool_name: Option<String>,
    pub status: RecoveredToolCallStatus,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveredToolCallStatus {
    Pending,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveredToolEffect {
    pub idempotency_key: String,
    pub call_id: String,
    pub tool_name: String,
    pub input_hash: String,
    pub completed_at: Timestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveredApprovalWait {
    pub approval_id: String,
    pub checkpoint_id: Option<CheckpointId>,
    pub action: String,
    pub resource_kind: String,
    pub resource: String,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveryDiagnostic {
    pub severity: RecoveryDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub source: Option<String>,
    pub truncated: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResumeDryRunReport {
    pub snapshot: RecoverySnapshot,
    pub will_replay_event_count: u32,
    pub will_resume_pending_tool_count: u32,
    pub will_reuse_idempotent_effect_count: u32,
    pub blocked: bool,
}
