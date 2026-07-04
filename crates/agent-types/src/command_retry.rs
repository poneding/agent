use serde::{Deserialize, Serialize};

use crate::Timestamp;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandRetryRequest {
    pub command_run_id: String,
    pub reason: String,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandRetryPlan {
    pub command_run_id: String,
    pub attempt: CommandRetryAttemptLink,
    pub readiness: CommandRetryReadiness,
    pub classification: CommandRetryFailureClassification,
    pub command: CommandRetryCommandSummary,
    pub exit_code: Option<i32>,
    pub reason: String,
    pub dry_run: bool,
    pub diagnostics: Vec<CommandRetryDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandRetryAttemptLink {
    pub previous_attempt_id: String,
    pub next_attempt_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandRetryCommandSummary {
    pub display: String,
    pub cwd: String,
    pub classify_as_test: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandRetryReadiness {
    Retryable,
    AlreadySucceeded,
    WaitingApproval,
    PolicyBlocked,
    StaleTarget,
    MissingEvidence,
    InvalidTarget,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandRetryFailureClassification {
    None,
    TestFailure,
    CommandFailure,
    Timeout,
    PolicyDenied,
    WaitingApproval,
    ToolFailure,
    InvalidInput,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommandRetryDiagnostic {
    pub severity: CommandRetryDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub truncated: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandRetryDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IdempotentRecoverySummary {
    pub reusable_effect_count: u32,
    pub pending_tool_count: u32,
    pub conflict_count: u32,
    pub effects: Vec<RecoveredToolEffectDisposition>,
    pub diagnostics: Vec<crate::RecoveryDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveredToolEffectDisposition {
    pub call_id: String,
    pub tool_name: String,
    pub idempotency_key: String,
    pub input_hash: String,
    pub disposition: ToolEffectDisposition,
    pub completed_at: Timestamp,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolEffectDisposition {
    Reusable,
    SkippedDuplicate,
    PendingReplay,
    Conflict,
}
