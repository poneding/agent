use serde::{Deserialize, Serialize};

use crate::approval_commands::{DesktopResolveApprovalRequest, resolve_approval_in_session};
use crate::bootstrap::{DesktopBootstrap, DesktopBootstrapError, build_desktop_bootstrap};
use crate::bridge::{DesktopCommandAck, DesktopSessionSnapshot, DesktopStartSessionRequest};
use crate::state::DesktopAppState;
use crate::workspace_snapshot::build_empty_workspace_snapshot_for_session;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopCommandError {
    pub code: String,
    pub message: String,
}

impl DesktopCommandError {
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: redact_secret_like_text(&message.into()),
        }
    }
}

impl From<DesktopBootstrapError> for DesktopCommandError {
    fn from(error: DesktopBootstrapError) -> Self {
        Self::new("desktop.bootstrap.invalid", error.to_string())
    }
}

#[tauri::command]
pub async fn desktop_bootstrap(
    state: tauri::State<'_, DesktopAppState>,
) -> Result<DesktopBootstrap, DesktopCommandError> {
    build_desktop_bootstrap(&state.app_name, &state.app_version, Vec::new()).map_err(Into::into)
}

#[tauri::command]
pub async fn start_session(
    state: tauri::State<'_, DesktopAppState>,
    request: DesktopStartSessionRequest,
) -> Result<DesktopSessionSnapshot, DesktopCommandError> {
    let mut session = state.runtime_session.lock().map_err(|_| {
        DesktopCommandError::new("desktop.bridge.lock_failed", "runtime session lock failed")
    })?;
    session.start(request)
}

#[tauri::command]
pub async fn stop_session(
    state: tauri::State<'_, DesktopAppState>,
) -> Result<DesktopSessionSnapshot, DesktopCommandError> {
    let mut session = state.runtime_session.lock().map_err(|_| {
        DesktopCommandError::new("desktop.bridge.lock_failed", "runtime session lock failed")
    })?;
    session.stop()
}

#[tauri::command]
pub async fn send_ui_command(
    state: tauri::State<'_, DesktopAppState>,
    envelope: agent_ui_bridge::UiCommandEnvelope,
) -> Result<DesktopCommandAck, DesktopCommandError> {
    let mut session = state.runtime_session.lock().map_err(|_| {
        DesktopCommandError::new("desktop.bridge.lock_failed", "runtime session lock failed")
    })?;
    session.send_ui_command(envelope)
}

#[tauri::command]
pub async fn drain_ui_frames(
    state: tauri::State<'_, DesktopAppState>,
    max_count: usize,
) -> Result<Vec<agent_ui_bridge::UiJsonFrame>, DesktopCommandError> {
    let mut session = state.runtime_session.lock().map_err(|_| {
        DesktopCommandError::new("desktop.bridge.lock_failed", "runtime session lock failed")
    })?;
    session.drain_ui_frames(max_count)
}

#[tauri::command]
pub async fn load_workspace_snapshot(
    state: tauri::State<'_, DesktopAppState>,
) -> Result<agent_desktop::DesktopWorkspaceSnapshot, DesktopCommandError> {
    let mut session = state.runtime_session.lock().map_err(|_| {
        DesktopCommandError::new("desktop.bridge.lock_failed", "runtime session lock failed")
    })?;
    build_empty_workspace_snapshot_for_session(&mut session)
}

#[tauri::command]
pub async fn resolve_approval(
    state: tauri::State<'_, DesktopAppState>,
    request: DesktopResolveApprovalRequest,
    client_version: String,
) -> Result<DesktopCommandAck, DesktopCommandError> {
    let mut session = state.runtime_session.lock().map_err(|_| {
        DesktopCommandError::new("desktop.bridge.lock_failed", "runtime session lock failed")
    })?;
    resolve_approval_in_session(&mut session, request, client_version)
}

pub(crate) fn redact_secret_like_text(value: &str) -> String {
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
