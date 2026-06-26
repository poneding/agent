use serde::{Deserialize, Serialize};

use crate::{UiBridgeError, UiEventEnvelope};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiConfigSnapshot {
    pub selected_provider: String,
    pub selected_model: String,
    pub providers: Vec<UiModelProviderSummary>,
    pub diagnostics: Vec<UiConfigDiagnostic>,
}

impl UiConfigSnapshot {
    pub fn new(
        selected_provider: impl Into<String>,
        selected_model: impl Into<String>,
        mut providers: Vec<UiModelProviderSummary>,
        diagnostics: Vec<UiConfigDiagnostic>,
    ) -> Result<Self, UiBridgeError> {
        let selected_provider = selected_provider.into();
        let selected_model = selected_model.into();
        validate_non_empty("selected provider", &selected_provider)?;
        validate_non_empty("selected model", &selected_model)?;
        providers.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(Self {
            selected_provider,
            selected_model,
            providers,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiModelProviderSummary {
    pub id: String,
    pub kind: String,
    pub default_model: String,
    pub enabled: bool,
}

impl UiModelProviderSummary {
    pub fn new(
        id: impl Into<String>,
        kind: impl Into<String>,
        default_model: impl Into<String>,
        enabled: bool,
    ) -> Result<Self, UiBridgeError> {
        let id = id.into();
        let kind = kind.into();
        let default_model = default_model.into();
        validate_non_empty("provider id", &id)?;
        validate_non_empty("provider kind", &kind)?;
        validate_non_empty("provider default model", &default_model)?;

        Ok(Self {
            id,
            kind,
            default_model,
            enabled,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiConfigDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiConfigDiagnostic {
    pub severity: UiConfigDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub field_path: Option<String>,
    pub source: Option<String>,
}

impl UiConfigDiagnostic {
    pub fn new(
        severity: UiConfigDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        field_path: Option<String>,
        source: Option<String>,
    ) -> Result<Self, UiBridgeError> {
        let code = code.into();
        let message = message.into();
        validate_non_empty("diagnostic code", &code)?;
        validate_non_empty("diagnostic message", &message)?;

        Ok(Self {
            severity,
            code,
            message,
            field_path,
            source,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiJsonFrame {
    Event(UiEventEnvelope),
    ConfigSnapshot(UiConfigSnapshot),
    ModelSelection {
        provider: String,
        model: String,
    },
    ConfigSummary {
        selected_provider: String,
        selected_model: String,
        provider_ids: Vec<String>,
        layer_count: u64,
        diagnostic_count: u64,
    },
    ConfigDiagnostic {
        severity: String,
        code: String,
        message: String,
        field_path: Option<String>,
        source: Option<String>,
    },
    Done {
        event_count: u64,
    },
    Error {
        code: String,
        message: String,
        recoverable: bool,
    },
}

pub fn encode_json_frame(frame: &UiJsonFrame) -> Result<String, UiBridgeError> {
    let mut line = serde_json::to_string(frame)?;
    line.push('\n');
    Ok(line)
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), UiBridgeError> {
    if value.trim().is_empty() {
        return Err(UiBridgeError::Validation(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(())
}
