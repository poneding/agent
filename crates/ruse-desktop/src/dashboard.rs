use ruse_types::{ConversationId, SessionId, TaskState, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::{DesktopConfigSnapshot, DesktopError, DesktopWorkspaceSnapshot};

const MAX_DASHBOARD_TEXT_CHARS: usize = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopDashboardInput {
    pub workspace: DesktopWorkspaceSnapshot,
    pub workspace_root_display: Option<String>,
    pub session: DesktopSessionReadiness,
    pub config: DesktopConfigSnapshot,
    pub pending_approval_count: u32,
    pub diagnostics: Vec<DesktopDashboardDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopDashboardSnapshot {
    pub workspace_id: WorkspaceId,
    pub workspace_root_display: Option<String>,
    pub session: DesktopSessionReadiness,
    pub provider: DesktopProviderReadiness,
    pub task_counts: DesktopTaskCounts,
    pub pending_approval_count: u32,
    pub diagnostics: Vec<DesktopDashboardDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopSessionReadiness {
    pub state: DesktopSessionReadinessState,
    pub session_id: Option<SessionId>,
    pub conversation_id: Option<ConversationId>,
    pub pending_frame_count: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopSessionReadinessState {
    NotStarted,
    Connecting,
    Connected,
    Disconnected,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopProviderReadiness {
    pub provider_id: String,
    pub model_id: String,
    pub enabled: bool,
    pub diagnostic_count: u32,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskCounts {
    pub total: u32,
    pub active: u32,
    pub waiting_approval: u32,
    pub failed: u32,
    pub terminal: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopDashboardDiagnostic {
    pub severity: DesktopDashboardDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub truncated: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopDashboardDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl DesktopDashboardDiagnostic {
    pub fn new(
        severity: DesktopDashboardDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, DesktopError> {
        let (code, code_truncated) = bounded_required_text("dashboard diagnostic code", code)?;
        let (message, message_truncated) =
            bounded_required_text("dashboard diagnostic message", message)?;
        Ok(Self {
            severity,
            code,
            message,
            truncated: code_truncated || message_truncated,
        })
    }
}

pub fn build_dashboard_snapshot(
    input: DesktopDashboardInput,
) -> Result<DesktopDashboardSnapshot, DesktopError> {
    let workspace_root_display = input
        .workspace_root_display
        .map(bounded_optional_text)
        .transpose()?
        .flatten();
    let provider = build_provider_readiness(&input.config)?;
    let task_counts = build_task_counts(&input.workspace);

    Ok(DesktopDashboardSnapshot {
        workspace_id: input.workspace.workspace_id,
        workspace_root_display,
        session: input.session,
        provider,
        task_counts,
        pending_approval_count: input.pending_approval_count,
        diagnostics: input.diagnostics,
    })
}

fn build_provider_readiness(
    config: &DesktopConfigSnapshot,
) -> Result<DesktopProviderReadiness, DesktopError> {
    let selected_provider = config.snapshot.selected_provider.as_str();
    let selected_model = config.snapshot.selected_model.as_str();
    let enabled = config
        .snapshot
        .providers
        .iter()
        .find(|provider| provider.id == selected_provider)
        .is_some_and(|provider| provider.enabled);
    let (provider_id, _) = bounded_required_text("selected provider", selected_provider)?;
    let (model_id, _) = bounded_required_text("selected model", selected_model)?;

    Ok(DesktopProviderReadiness {
        provider_id,
        model_id,
        enabled,
        diagnostic_count: usize_to_u32(config.snapshot.diagnostics.len()),
    })
}

fn build_task_counts(workspace: &DesktopWorkspaceSnapshot) -> DesktopTaskCounts {
    let mut counts = DesktopTaskCounts {
        total: usize_to_u32(workspace.tasks.len()),
        ..DesktopTaskCounts::default()
    };

    for task in &workspace.tasks {
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

fn bounded_required_text(
    label: &str,
    value: impl Into<String>,
) -> Result<(String, bool), DesktopError> {
    let value = value.into();
    let value = value.trim();
    if value.is_empty() {
        return Err(DesktopError::InvalidSnapshot(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(bound_dashboard_text(value))
}

fn bounded_optional_text(value: String) -> Result<Option<String>, DesktopError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(bound_dashboard_text(value).0))
}

fn bound_dashboard_text(value: &str) -> (String, bool) {
    let redacted = redact_secret_like_text(value);
    let redacted = redacted.trim();
    if redacted.chars().count() > MAX_DASHBOARD_TEXT_CHARS {
        (
            redacted.chars().take(MAX_DASHBOARD_TEXT_CHARS).collect(),
            true,
        )
    } else {
        (redacted.to_string(), false)
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
