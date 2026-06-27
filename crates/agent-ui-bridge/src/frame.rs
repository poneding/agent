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
pub struct UiPromptSummary {
    pub profile_id: String,
    pub profile_version: u32,
    pub display_name: String,
    pub bundle_hash: String,
    pub estimated_tokens: u32,
    pub message_count: u32,
    pub layers: Vec<UiPromptLayerSummary>,
    pub diagnostics: Vec<UiPromptDiagnosticSummary>,
}

impl UiPromptSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profile_id: impl Into<String>,
        profile_version: u32,
        display_name: impl Into<String>,
        bundle_hash: impl Into<String>,
        estimated_tokens: u32,
        message_count: u32,
        layers: Vec<UiPromptLayerSummary>,
        diagnostics: Vec<UiPromptDiagnosticSummary>,
    ) -> Result<Self, UiBridgeError> {
        let profile_id = profile_id.into();
        let display_name = display_name.into();
        let bundle_hash = bundle_hash.into();
        validate_non_empty("prompt profile id", &profile_id)?;
        validate_non_empty("prompt display name", &display_name)?;
        validate_non_empty("prompt bundle hash", &bundle_hash)?;
        if profile_version == 0 {
            return Err(UiBridgeError::Validation(
                "prompt profile version must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            profile_id,
            profile_version,
            display_name,
            bundle_hash,
            estimated_tokens,
            message_count,
            layers,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPromptLayerSummary {
    pub layer_id: String,
    pub kind: String,
    pub source_label: String,
    pub priority: i32,
    pub token_estimate: u32,
    pub visibility: String,
    pub sensitivity: String,
    pub content_preview: Option<String>,
    pub source_hash: Option<String>,
    pub content_hash: Option<String>,
}

impl UiPromptLayerSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layer_id: impl Into<String>,
        kind: impl Into<String>,
        source_label: impl Into<String>,
        priority: i32,
        token_estimate: u32,
        visibility: impl Into<String>,
        sensitivity: impl Into<String>,
        content_preview: Option<String>,
        source_hash: Option<String>,
        content_hash: Option<String>,
    ) -> Result<Self, UiBridgeError> {
        let layer_id = layer_id.into();
        let kind = kind.into();
        let source_label = source_label.into();
        let visibility = visibility.into();
        let sensitivity = sensitivity.into();
        validate_non_empty("prompt layer id", &layer_id)?;
        validate_non_empty("prompt layer kind", &kind)?;
        validate_non_empty("prompt layer source label", &source_label)?;
        validate_non_empty("prompt layer visibility", &visibility)?;
        validate_non_empty("prompt layer sensitivity", &sensitivity)?;

        let content_preview = if should_hide_prompt_preview(&visibility, &sensitivity)
            || content_preview
                .as_deref()
                .is_some_and(contains_secret_like_text)
        {
            None
        } else {
            content_preview.and_then(|preview| {
                let preview = preview.trim();
                if preview.is_empty() {
                    None
                } else {
                    Some(preview.to_string())
                }
            })
        };

        Ok(Self {
            layer_id,
            kind,
            source_label,
            priority,
            token_estimate,
            visibility,
            sensitivity,
            content_preview,
            source_hash: normalize_optional_string(source_hash),
            content_hash: normalize_optional_string(content_hash),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiPromptDiagnosticSummary {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub layer_id: Option<String>,
}

impl UiPromptDiagnosticSummary {
    pub fn new(
        severity: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        layer_id: Option<String>,
    ) -> Result<Self, UiBridgeError> {
        let severity = severity.into();
        let code = code.into();
        let message = message.into();
        validate_non_empty("prompt diagnostic severity", &severity)?;
        validate_non_empty("prompt diagnostic code", &code)?;
        validate_non_empty("prompt diagnostic message", &message)?;

        Ok(Self {
            severity,
            code,
            message,
            layer_id: normalize_optional_string(layer_id),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionSnapshot {
    pub extensions: Vec<UiExtensionSummary>,
    pub capability_reviews: Vec<UiExtensionCapabilityReview>,
    pub diagnostics: Vec<UiExtensionDiagnostic>,
}

impl UiExtensionSnapshot {
    pub fn new(
        mut extensions: Vec<UiExtensionSummary>,
        capability_reviews: Vec<UiExtensionCapabilityReview>,
        diagnostics: Vec<UiExtensionDiagnostic>,
    ) -> Result<Self, UiBridgeError> {
        extensions.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(Self {
            extensions,
            capability_reviews,
            diagnostics,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionSummary {
    pub id: String,
    pub kind: String,
    pub enabled: bool,
    pub health: String,
    pub version: Option<String>,
    pub source_label: String,
    pub capability_count: u32,
}

impl UiExtensionSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        kind: impl Into<String>,
        enabled: bool,
        health: impl Into<String>,
        version: Option<String>,
        source_label: impl Into<String>,
        capability_count: u32,
    ) -> Result<Self, UiBridgeError> {
        let id = id.into();
        let kind = kind.into();
        let health = health.into();
        let source_label = source_label.into();
        validate_non_empty("extension id", &id)?;
        validate_non_empty("extension kind", &kind)?;
        validate_non_empty("extension health", &health)?;
        validate_non_empty("extension source label", &source_label)?;

        Ok(Self {
            id,
            kind,
            enabled,
            health: redact_secret_like_text(&health),
            version: normalize_optional_string(version)
                .map(|value| redact_secret_like_text(&value)),
            source_label: redact_secret_like_text(&source_label),
            capability_count,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionCapabilityReview {
    pub extension_id: String,
    pub capability_kind: String,
    pub resource: String,
    pub reason: String,
}

impl UiExtensionCapabilityReview {
    pub fn new(
        extension_id: impl Into<String>,
        capability_kind: impl Into<String>,
        resource: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self, UiBridgeError> {
        let extension_id = extension_id.into();
        let capability_kind = capability_kind.into();
        let resource = resource.into();
        let reason = reason.into();
        validate_non_empty("extension review id", &extension_id)?;
        validate_non_empty("extension review capability kind", &capability_kind)?;
        validate_non_empty("extension review resource", &resource)?;
        validate_non_empty("extension review reason", &reason)?;

        Ok(Self {
            extension_id,
            capability_kind: redact_secret_like_text(&capability_kind),
            resource: redact_secret_like_text(&resource),
            reason: redact_secret_like_text(&reason),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiExtensionDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub extension_id: Option<String>,
    pub truncated: bool,
}

impl UiExtensionDiagnostic {
    pub fn new(
        severity: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        extension_id: Option<String>,
        truncated: bool,
    ) -> Result<Self, UiBridgeError> {
        let severity = severity.into();
        let code = code.into();
        let message = message.into();
        validate_non_empty("extension diagnostic severity", &severity)?;
        validate_non_empty("extension diagnostic code", &code)?;
        validate_non_empty("extension diagnostic message", &message)?;

        Ok(Self {
            severity: redact_secret_like_text(&severity),
            code,
            message: redact_secret_like_text(&message),
            extension_id: normalize_optional_string(extension_id),
            truncated,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiJsonFrame {
    Event(UiEventEnvelope),
    ConfigSnapshot(UiConfigSnapshot),
    PromptSummary(UiPromptSummary),
    ExtensionSnapshot(UiExtensionSnapshot),
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

fn should_hide_prompt_preview(visibility: &str, sensitivity: &str) -> bool {
    visibility.eq_ignore_ascii_case("hidden") || sensitivity.eq_ignore_ascii_case("secret_like")
}

fn contains_secret_like_text(value: &str) -> bool {
    value.split_whitespace().any(is_secret_like_token)
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            if is_secret_like_token(part) {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like_token(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}
