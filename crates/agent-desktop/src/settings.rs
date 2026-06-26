use serde::{Deserialize, Serialize};

use crate::DesktopError;
use agent_ui_bridge::UiConfigSnapshot;

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
