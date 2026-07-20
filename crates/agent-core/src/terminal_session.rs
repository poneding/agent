use std::collections::{BTreeMap, BTreeSet};

use agent_storage::{EventStore, StorageError};
use agent_types::{
    ClientKind, ConversationId, RuntimeEvent, RuntimeEventPayload, Timestamp, WorkspaceId,
};
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

#[derive(Clone, Debug, Default)]
pub struct TerminalSessionCatalog {
    records: BTreeMap<(WorkspaceId, ConversationId), TerminalSessionRecord>,
    titled_conversations: BTreeSet<ConversationId>,
}

impl TerminalSessionCatalog {
    #[must_use]
    pub fn new(records: Vec<TerminalSessionRecord>) -> Self {
        let mut catalog = Self::default();
        for record in records {
            if !is_placeholder_title(&record.title) {
                catalog.titled_conversations.insert(record.conversation_id);
            }
            catalog
                .records
                .insert((record.workspace_id, record.conversation_id), record);
        }
        catalog
    }

    #[must_use]
    pub fn from_report(report: &TerminalSessionDirectoryReport) -> Self {
        Self::new(report.sessions.iter().map(record_from_summary).collect())
    }

    pub fn hydrate_from_store<E>(&mut self, store: &E) -> Result<(), StorageError>
    where
        E: EventStore + ?Sized,
    {
        for conversation_id in store.list_conversation_ids()? {
            for stored in store.list_by_conversation(conversation_id)? {
                self.apply_event(&stored.event);
            }
        }
        Ok(())
    }

    pub fn report(&self) -> Result<TerminalSessionDirectoryReport, TerminalSessionDirectoryError> {
        build_terminal_session_directory(self.records.values().cloned().collect())
    }

    #[must_use]
    pub fn start_new_conversation(
        &mut self,
        current: ConversationId,
        conversation_id: ConversationId,
        created_at: Timestamp,
    ) -> Option<TerminalSessionRecord> {
        let mut record = self
            .records
            .values()
            .find(|record| record.conversation_id == current)
            .cloned()
            .or_else(|| self.records.values().next().cloned())?;
        record.conversation_id = conversation_id;
        record.title = "New session".to_string();
        record.last_event_at = created_at;
        record.pending_approval_count = 0;
        record.running_command_count = 0;
        record.task_count = 0;
        record.failed = false;
        self.records.insert(
            (record.workspace_id, record.conversation_id),
            record.clone(),
        );
        Some(record)
    }

    pub fn apply_event(&mut self, event: &RuntimeEvent) {
        let key = (event.workspace_id, event.conversation_id);
        if !self.records.contains_key(&key) {
            let mut record = self
                .records
                .values()
                .find(|record| record.workspace_id == event.workspace_id)
                .cloned()
                .or_else(|| self.records.values().next().cloned())
                .unwrap_or_else(|| record_from_event(event));
            record.workspace_id = event.workspace_id;
            record.conversation_id = event.conversation_id;
            record.title = "New session".to_string();
            record.last_event_at = event.occurred_at;
            record.pending_approval_count = 0;
            record.running_command_count = 0;
            record.task_count = 0;
            record.failed = false;
            self.records.insert(key, record);
        }

        let Some(record) = self.records.get_mut(&key) else {
            return;
        };
        record.last_event_at = event.occurred_at;
        match &event.payload {
            RuntimeEventPayload::ConversationStarted(started) => {
                record.title = bounded_public_text(&started.title);
                if is_placeholder_title(&record.title) {
                    self.titled_conversations.remove(&event.conversation_id);
                } else {
                    self.titled_conversations.insert(event.conversation_id);
                }
            }
            RuntimeEventPayload::UserInputReceived(input)
                if !self.titled_conversations.contains(&event.conversation_id) =>
            {
                record.title = summarize_terminal_session_title(&input.message);
                self.titled_conversations.insert(event.conversation_id);
            }
            RuntimeEventPayload::ModelStreamStarted(started) => {
                record.provider_id = bounded_public_text(&started.provider);
                record.model_id = bounded_public_text(&started.model);
            }
            RuntimeEventPayload::TurnFailed(_) => record.failed = true,
            RuntimeEventPayload::RuntimeError(error) if !error.recoverable => {
                record.failed = true;
            }
            _ => {}
        }
    }
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

fn record_from_event(event: &RuntimeEvent) -> TerminalSessionRecord {
    TerminalSessionRecord {
        workspace_id: event.workspace_id,
        conversation_id: event.conversation_id,
        workspace_root_display: ".".to_string(),
        title: "New session".to_string(),
        client_kind: ClientKind::Cli,
        client_version: "unknown".to_string(),
        provider_id: "unknown".to_string(),
        model_id: "unknown".to_string(),
        git_state: RepoGitState::Unavailable,
        last_event_at: event.occurred_at,
        pending_approval_count: 0,
        running_command_count: 0,
        task_count: 0,
        failed: false,
    }
}

fn record_from_summary(summary: &TerminalSessionSummary) -> TerminalSessionRecord {
    TerminalSessionRecord {
        workspace_id: summary.workspace_id,
        conversation_id: summary.conversation_id,
        workspace_root_display: summary.workspace_root_display.clone(),
        title: summary.title.clone(),
        client_kind: summary.client_kind.clone(),
        client_version: summary.client_version.clone(),
        provider_id: summary.provider_id.clone(),
        model_id: summary.model_id.clone(),
        git_state: summary.git_state,
        last_event_at: summary.last_event_at,
        pending_approval_count: summary.pending_approval_count,
        running_command_count: summary.running_command_count,
        task_count: summary.task_count,
        failed: summary.status == TerminalSessionStatus::Failed,
    }
}

#[must_use]
pub fn summarize_terminal_session_title(message: &str) -> String {
    let redacted = redact_secret_like_text(message);
    let normalized = redacted.split_whitespace().collect::<Vec<_>>().join(" ");
    let sentence = normalized
        .split(['\n', '.', '!', '?', '。', '！', '？'])
        .next()
        .unwrap_or_default()
        .trim();
    if sentence.is_empty() {
        return "New session".to_string();
    }

    let mut title = sentence.chars().take(56).collect::<String>();
    if sentence.chars().count() > 56 {
        title.push_str("...");
    }
    title
}

fn is_placeholder_title(title: &str) -> bool {
    matches!(title.trim(), "" | "Repo session" | "New session")
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
