use std::collections::BTreeMap;

use agent_types::{ClientKind, ConversationId, Timestamp, WorkspaceId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::RepoGitState;

const PUBLIC_TEXT_LIMIT: usize = 160;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TerminalSessionRecord {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub workspace_root_display: String,
    pub title: String,
    pub client_kind: ClientKind,
    pub client_version: String,
    pub provider_id: String,
    pub model_id: String,
    pub git_state: RepoGitState,
    pub last_event_at: Timestamp,
    pub pending_approval_count: u32,
    pub running_command_count: u32,
    pub task_count: u32,
    pub failed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TerminalSessionDirectoryReport {
    pub sessions: Vec<TerminalSessionSummary>,
    pub diagnostics: Vec<TerminalSessionDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TerminalSessionSummary {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub workspace_root_display: String,
    pub title: String,
    pub client_kind: ClientKind,
    pub client_version: String,
    pub provider_id: String,
    pub model_id: String,
    pub git_state: RepoGitState,
    pub last_event_at: Timestamp,
    pub status: TerminalSessionStatus,
    pub pending_approval_count: u32,
    pub running_command_count: u32,
    pub task_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalSessionStatus {
    Idle,
    HasTasks,
    RunningCommand,
    WaitingApproval,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TerminalSessionDiagnostic {
    pub severity: TerminalSessionDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalSessionDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TerminalSessionDirectoryError {
    #[error("terminal session directory is invalid")]
    Invalid {
        diagnostics: Vec<TerminalSessionDiagnostic>,
    },
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TerminalSessionRestoreError {
    #[error("terminal session restore target was not found: {conversation_id}")]
    NotFound { conversation_id: ConversationId },
}

impl TerminalSessionRestoreError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "terminal_session.restore.not_found",
        }
    }
}

#[must_use]
pub fn terminal_session_status(record: &TerminalSessionRecord) -> TerminalSessionStatus {
    if record.failed {
        TerminalSessionStatus::Failed
    } else if record.pending_approval_count > 0 {
        TerminalSessionStatus::WaitingApproval
    } else if record.running_command_count > 0 {
        TerminalSessionStatus::RunningCommand
    } else if record.task_count > 0 {
        TerminalSessionStatus::HasTasks
    } else {
        TerminalSessionStatus::Idle
    }
}

pub fn build_terminal_session_directory(
    records: Vec<TerminalSessionRecord>,
) -> Result<TerminalSessionDirectoryReport, TerminalSessionDirectoryError> {
    let mut latest_by_session =
        BTreeMap::<(WorkspaceId, ConversationId), TerminalSessionRecord>::new();
    for record in records {
        let key = (record.workspace_id, record.conversation_id);
        match latest_by_session.get(&key) {
            Some(existing) if existing.last_event_at >= record.last_event_at => {}
            _ => {
                latest_by_session.insert(key, record);
            }
        }
    }

    let mut sessions = latest_by_session
        .into_values()
        .map(safe_summary)
        .collect::<Vec<_>>();
    sessions.sort_by(|left, right| {
        right
            .last_event_at
            .cmp(&left.last_event_at)
            .then_with(|| {
                left.workspace_root_display
                    .cmp(&right.workspace_root_display)
            })
            .then_with(|| left.conversation_id.cmp(&right.conversation_id))
    });

    Ok(TerminalSessionDirectoryReport {
        sessions,
        diagnostics: Vec::new(),
    })
}

pub fn validate_terminal_session_restore(
    report: &TerminalSessionDirectoryReport,
    conversation_id: ConversationId,
) -> Result<TerminalSessionSummary, TerminalSessionRestoreError> {
    report
        .sessions
        .iter()
        .find(|session| session.conversation_id == conversation_id)
        .cloned()
        .ok_or(TerminalSessionRestoreError::NotFound { conversation_id })
}

fn safe_summary(record: TerminalSessionRecord) -> TerminalSessionSummary {
    let status = terminal_session_status(&record);
    TerminalSessionSummary {
        workspace_id: record.workspace_id,
        conversation_id: record.conversation_id,
        workspace_root_display: bounded_public_text(&record.workspace_root_display),
        title: bounded_public_text(&record.title),
        client_kind: record.client_kind,
        client_version: bounded_public_text(&record.client_version),
        provider_id: bounded_public_text(&record.provider_id),
        model_id: bounded_public_text(&record.model_id),
        git_state: record.git_state,
        last_event_at: record.last_event_at,
        status,
        pending_approval_count: record.pending_approval_count,
        running_command_count: record.running_command_count,
        task_count: record.task_count,
    }
}

fn bounded_public_text(value: &str) -> String {
    let redacted = redact_secret_like_text(value);
    let trimmed = redacted.trim();
    if trimmed.chars().count() > PUBLIC_TEXT_LIMIT {
        trimmed.chars().take(PUBLIC_TEXT_LIMIT).collect()
    } else {
        trimmed.to_string()
    }
}

fn redact_secret_like_text(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            if is_secret_like(word) {
                "[redacted]"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}
