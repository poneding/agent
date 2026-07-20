use ruse_desktop::{
    DesktopConfigSnapshot, DesktopError, DesktopSettings, validate_desktop_settings,
};
use ruse_ui_bridge::{UiConfigDiagnostic, UiConfigSnapshot};

use crate::{
    bootstrap::{DesktopBootstrapDiagnostic, DesktopBootstrapError},
    commands::{DesktopCommandError, redact_secret_like_text},
    state::DesktopAppState,
};

#[tauri::command]
pub async fn load_config_snapshot(
    state: tauri::State<'_, DesktopAppState>,
) -> Result<DesktopConfigSnapshot, DesktopCommandError> {
    let snapshot = state.config_snapshot.lock().map_err(|_| {
        DesktopCommandError::new(
            "desktop.settings.lock_failed",
            "config snapshot lock failed",
        )
    })?;
    build_config_snapshot_response(snapshot.snapshot.clone())
}

#[tauri::command]
pub async fn validate_settings(
    settings: DesktopSettings,
) -> Result<Vec<DesktopBootstrapDiagnostic>, DesktopCommandError> {
    validate_settings_request(settings)
}

pub(crate) fn build_config_snapshot_response(
    snapshot: UiConfigSnapshot,
) -> Result<DesktopConfigSnapshot, DesktopCommandError> {
    let snapshot = UiConfigSnapshot {
        diagnostics: snapshot
            .diagnostics
            .into_iter()
            .map(redact_config_diagnostic)
            .collect(),
        ..snapshot
    };
    DesktopConfigSnapshot::new(snapshot).map_err(settings_error)
}

pub(crate) fn validate_settings_request(
    settings: DesktopSettings,
) -> Result<Vec<DesktopBootstrapDiagnostic>, DesktopCommandError> {
    match validate_desktop_settings(&settings) {
        Ok(()) => Ok(Vec::new()),
        Err(error) => Ok(vec![settings_diagnostic(error)?]),
    }
}

fn redact_config_diagnostic(diagnostic: UiConfigDiagnostic) -> UiConfigDiagnostic {
    UiConfigDiagnostic {
        message: redact_secret_like_text(&diagnostic.message),
        ..diagnostic
    }
}

fn settings_diagnostic(
    error: DesktopError,
) -> Result<DesktopBootstrapDiagnostic, DesktopCommandError> {
    DesktopBootstrapDiagnostic::new(
        "error",
        "desktop.settings.invalid",
        redact_secret_like_text(&error.to_string()),
    )
    .map_err(bootstrap_error)
}

fn settings_error(error: DesktopError) -> DesktopCommandError {
    DesktopCommandError::new("desktop.settings.invalid", error.to_string())
}

fn bootstrap_error(error: DesktopBootstrapError) -> DesktopCommandError {
    DesktopCommandError::new("desktop.settings.invalid", error.to_string())
}
