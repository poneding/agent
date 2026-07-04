use std::path::PathBuf;

use agent_core::{PlatformPathOverrides, PlatformPaths, ReleaseChannel, SettingsProfile};
use serde::{Deserialize, Serialize};

use crate::DesktopError;
use agent_ui_bridge::{UiConfigDiagnosticSeverity, UiConfigSnapshot};

const MAX_SETTINGS_TEXT_CHARS: usize = 240;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopSettings {
    pub model: ModelSettings,
    pub policy: PolicySettings,
    pub memory: MemorySettings,
    pub mcp: McpSettings,
    pub ui: UiSettings,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelSettings {
    pub provider: String,
    pub model: String,
}

impl Default for ModelSettings {
    fn default() -> Self {
        Self {
            provider: "default".to_string(),
            model: "default".to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicySettings {
    pub default_decision: String,
    pub require_approval_for_shell: bool,
}

impl Default for PolicySettings {
    fn default() -> Self {
        Self {
            default_decision: "ask".to_string(),
            require_approval_for_shell: true,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemorySettings {
    pub project_memory_enabled: bool,
    pub long_term_memory_enabled: bool,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpSettings {
    pub enabled: bool,
    pub server_count: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopTheme {
    System,
    Light,
    Dark,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: DesktopTheme,
    pub show_trace_timeline: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: DesktopTheme::System,
            show_trace_timeline: true,
        }
    }
}

pub fn validate_desktop_settings(settings: &DesktopSettings) -> Result<(), DesktopError> {
    validate_non_empty("model provider", &settings.model.provider)?;
    validate_non_empty("model name", &settings.model.model)?;
    validate_policy_decision(&settings.policy.default_decision)?;
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopProviderSettingsSummary {
    pub selected_provider: String,
    pub selected_model: String,
    pub providers: Vec<DesktopProviderSummary>,
    pub diagnostics: Vec<DesktopSettingsDiagnostic>,
    pub validation: DesktopSettingsValidationSummary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopProviderSummary {
    pub id: String,
    pub kind: String,
    pub default_model: String,
    pub enabled: bool,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_structured_outputs: bool,
    pub supports_image_input: bool,
    pub context_window: Option<u32>,
    pub output_limit: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopSettingsDiagnostic {
    pub severity: DesktopSettingsDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub field_path: Option<String>,
    pub source: Option<String>,
    pub truncated: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopSettingsDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopSettingsValidationSummary {
    pub valid: bool,
    pub diagnostic_count: u32,
}

pub fn build_provider_settings_summary(
    config: &DesktopConfigSnapshot,
    settings: &DesktopSettings,
) -> Result<DesktopProviderSettingsSummary, DesktopError> {
    let providers = config
        .snapshot
        .providers
        .iter()
        .map(DesktopProviderSummary::from_ui_provider)
        .collect::<Result<Vec<_>, _>>()?;
    let mut diagnostics = config
        .snapshot
        .diagnostics
        .iter()
        .map(DesktopSettingsDiagnostic::from_ui_diagnostic)
        .collect::<Result<Vec<_>, _>>()?;

    if let Err(error) = validate_desktop_settings(settings) {
        diagnostics.push(DesktopSettingsDiagnostic::error(
            "desktop.settings.invalid",
            error.to_string(),
            None,
            Some("desktop".to_string()),
        )?);
    }

    Ok(DesktopProviderSettingsSummary {
        selected_provider: bounded_required_settings_text(
            "selected provider",
            config.snapshot.selected_provider.clone(),
        )?
        .0,
        selected_model: bounded_required_settings_text(
            "selected model",
            config.snapshot.selected_model.clone(),
        )?
        .0,
        providers,
        validation: DesktopSettingsValidationSummary {
            valid: diagnostics
                .iter()
                .all(|diagnostic| diagnostic.severity != DesktopSettingsDiagnosticSeverity::Error),
            diagnostic_count: usize_to_u32(diagnostics.len()),
        },
        diagnostics,
    })
}

impl DesktopProviderSummary {
    fn from_ui_provider(
        provider: &agent_ui_bridge::UiModelProviderSummary,
    ) -> Result<Self, DesktopError> {
        Ok(Self {
            id: bounded_required_settings_text("provider id", provider.id.clone())?.0,
            kind: bounded_required_settings_text("provider kind", provider.kind.clone())?.0,
            default_model: bounded_required_settings_text(
                "provider default model",
                provider.default_model.clone(),
            )?
            .0,
            enabled: provider.enabled,
            supports_tools: provider.capabilities.tool_calls,
            supports_streaming: provider.capabilities.streaming,
            supports_structured_outputs: provider.capabilities.structured_outputs,
            supports_image_input: provider.capabilities.image_input,
            context_window: provider.capabilities.max_context_tokens,
            output_limit: provider.capabilities.max_output_tokens,
        })
    }
}

impl DesktopSettingsDiagnostic {
    fn from_ui_diagnostic(
        diagnostic: &agent_ui_bridge::UiConfigDiagnostic,
    ) -> Result<Self, DesktopError> {
        let (code, code_truncated) =
            bounded_required_settings_text("settings diagnostic code", diagnostic.code.clone())?;
        let (message, message_truncated) = bounded_required_settings_text(
            "settings diagnostic message",
            diagnostic.message.clone(),
        )?;
        let field_path = diagnostic
            .field_path
            .clone()
            .map(|value| {
                bounded_optional_settings_text(value).map(|bounded| bounded.map(|item| item.0))
            })
            .transpose()?
            .flatten();
        let source = diagnostic
            .source
            .clone()
            .map(|value| {
                bounded_optional_settings_text(value).map(|bounded| bounded.map(|item| item.0))
            })
            .transpose()?
            .flatten();
        Ok(Self {
            severity: match diagnostic.severity {
                UiConfigDiagnosticSeverity::Info => DesktopSettingsDiagnosticSeverity::Info,
                UiConfigDiagnosticSeverity::Warning => DesktopSettingsDiagnosticSeverity::Warning,
                UiConfigDiagnosticSeverity::Error => DesktopSettingsDiagnosticSeverity::Error,
            },
            code,
            message,
            field_path,
            source,
            truncated: code_truncated || message_truncated,
        })
    }

    fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        field_path: Option<String>,
        source: Option<String>,
    ) -> Result<Self, DesktopError> {
        let (code, code_truncated) =
            bounded_required_settings_text("settings diagnostic code", code)?;
        let (message, message_truncated) =
            bounded_required_settings_text("settings diagnostic message", message)?;
        Ok(Self {
            severity: DesktopSettingsDiagnosticSeverity::Error,
            code,
            message,
            field_path,
            source,
            truncated: code_truncated || message_truncated,
        })
    }
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), DesktopError> {
    if value.trim().is_empty() {
        return Err(DesktopError::InvalidSettings(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(())
}

fn validate_policy_decision(value: &str) -> Result<(), DesktopError> {
    match value.trim() {
        "ask" | "allow" | "deny" => Ok(()),
        other => Err(DesktopError::InvalidSettings(format!(
            "unsupported policy default decision: {other}"
        ))),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopConfigSnapshot {
    pub snapshot: UiConfigSnapshot,
}

impl DesktopConfigSnapshot {
    pub fn new(snapshot: UiConfigSnapshot) -> Result<Self, DesktopError> {
        validate_config_selection("selected provider", &snapshot.selected_provider)?;
        validate_config_selection("selected model", &snapshot.selected_model)?;
        Ok(Self { snapshot })
    }
}

fn validate_config_selection(label: &str, value: &str) -> Result<(), DesktopError> {
    if value.trim().is_empty() {
        return Err(DesktopError::InvalidSnapshot(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(())
}

fn bounded_required_settings_text(
    label: &str,
    value: impl Into<String>,
) -> Result<(String, bool), DesktopError> {
    let value = value.into();
    let value = value.trim();
    if value.is_empty() {
        return Err(DesktopError::InvalidSettings(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(bound_settings_text(value))
}

fn bounded_optional_settings_text(value: String) -> Result<Option<(String, bool)>, DesktopError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(bound_settings_text(value)))
}

fn bound_settings_text(value: &str) -> (String, bool) {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() > MAX_SETTINGS_TEXT_CHARS {
        (
            redacted.chars().take(MAX_SETTINGS_TEXT_CHARS).collect(),
            true,
        )
    } else {
        (redacted, false)
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopReleaseSmokeSnapshot {
    pub app_version: String,
    pub release_channel: ReleaseChannel,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub settings_profile_id: String,
}

pub fn build_desktop_release_smoke_snapshot(
    app_version: impl Into<String>,
    release_channel: ReleaseChannel,
    portable_root: PathBuf,
) -> Result<DesktopReleaseSmokeSnapshot, DesktopError> {
    let app_version = app_version.into();
    if app_version.trim().is_empty() {
        return Err(DesktopError::InvalidSettings(
            "app version cannot be empty".to_string(),
        ));
    }

    let paths = PlatformPaths::current(PlatformPathOverrides {
        portable_root: Some(portable_root),
        ..PlatformPathOverrides::default()
    })
    .map_err(|error| DesktopError::InvalidSettings(error.to_string()))?;
    let settings = SettingsProfile {
        release_channel,
        ..SettingsProfile::default()
    };

    Ok(DesktopReleaseSmokeSnapshot {
        app_version,
        release_channel,
        config_dir: paths.config_dir,
        data_dir: paths.data_dir,
        cache_dir: paths.cache_dir,
        log_dir: paths.log_dir,
        settings_profile_id: settings.profile_id,
    })
}
