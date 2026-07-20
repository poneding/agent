use ruse_desktop::DesktopRoute;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRuntimeState {
    NotStarted,
    Connecting,
    Connected,
    Disconnected,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopBootstrap {
    pub app_name: String,
    pub app_version: String,
    pub runtime_state: DesktopRuntimeState,
    pub route: DesktopRoute,
    pub diagnostics: Vec<DesktopBootstrapDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopBootstrapDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
}

impl DesktopBootstrapDiagnostic {
    pub fn new(
        severity: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Result<Self, DesktopBootstrapError> {
        Ok(Self {
            severity: validate_non_empty("diagnostic severity", severity.into())?,
            code: validate_non_empty("diagnostic code", code.into())?,
            message: redact_secret_like_tokens(&validate_non_empty(
                "diagnostic message",
                message.into(),
            )?),
        })
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DesktopBootstrapError {
    #[error("invalid bootstrap field: {0}")]
    InvalidField(String),
}

pub fn build_desktop_bootstrap(
    app_name: impl Into<String>,
    app_version: impl Into<String>,
    diagnostics: Vec<DesktopBootstrapDiagnostic>,
) -> Result<DesktopBootstrap, DesktopBootstrapError> {
    Ok(DesktopBootstrap {
        app_name: validate_non_empty("app name", app_name.into())?,
        app_version: validate_non_empty("app version", app_version.into())?,
        runtime_state: DesktopRuntimeState::NotStarted,
        route: DesktopRoute::Dashboard,
        diagnostics,
    })
}

fn validate_non_empty(label: &str, value: String) -> Result<String, DesktopBootstrapError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DesktopBootstrapError::InvalidField(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(value)
}

fn redact_secret_like_tokens(value: &str) -> String {
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
