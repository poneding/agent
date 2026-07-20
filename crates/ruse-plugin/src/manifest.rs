use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use ruse_tool::ToolName;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::capability::{PluginCapabilityKind, PluginPermission};
use crate::compatibility::{PLUGIN_MANIFEST_SCHEMA_VERSION, check_plugin_compatibility};
use crate::error::PluginManifestError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PluginName(String);

impl PluginName {
    pub fn new(value: impl Into<String>) -> Result<Self, PluginManifestError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(PluginManifestError::InvalidName(
                "plugin name cannot be empty".to_string(),
            ));
        }

        if !value.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '.')
        }) {
            return Err(PluginManifestError::InvalidName(value));
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PluginName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PluginName {
    type Err = PluginManifestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for PluginName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PluginName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub schema_version: u32,
    pub name: PluginName,
    pub version: String,
    pub runtime_compatibility: PluginRuntimeCompatibility,
    pub isolation: PluginIsolation,
    pub capabilities: Vec<PluginCapabilityManifest>,
    pub tools: Vec<PluginToolManifest>,
    pub hooks: Vec<PluginHookManifest>,
    pub configuration_schema: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeCompatibility {
    pub min_agent_version: String,
    pub api_version: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginIsolationMode {
    Process,
    Wasm,
    Builtin,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginIsolation {
    pub mode: PluginIsolationMode,
    pub entrypoint: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginCapabilityManifest {
    pub kind: PluginCapabilityKind,
    pub name: String,
    pub permissions: Vec<PluginPermission>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginToolManifest {
    pub name: ToolName,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginHookManifest {
    pub name: String,
    pub event: String,
}

pub fn parse_plugin_manifest_json(input: &str) -> Result<PluginManifest, PluginManifestError> {
    let manifest: PluginManifest =
        serde_json::from_str(input).map_err(|error| classify_parse_error(error.to_string()))?;
    validate_schema_version(manifest.schema_version)?;
    Ok(manifest)
}

pub fn validate_plugin_manifest(manifest: &PluginManifest) -> Result<(), PluginManifestError> {
    validate_schema_version(manifest.schema_version)?;
    check_plugin_compatibility(&manifest.runtime_compatibility)?;
    validate_non_empty("plugin version", &manifest.version)?;
    validate_isolation(&manifest.isolation)?;
    validate_capabilities(&manifest.capabilities)?;
    validate_tools(&manifest.tools)?;
    validate_hooks(&manifest.hooks)?;

    if !manifest.configuration_schema.is_object() {
        return Err(PluginManifestError::InvalidManifest(
            "configuration_schema must be an object".to_string(),
        ));
    }

    Ok(())
}

fn validate_schema_version(schema_version: u32) -> Result<(), PluginManifestError> {
    if schema_version != PLUGIN_MANIFEST_SCHEMA_VERSION {
        return Err(PluginManifestError::UnsupportedSchema {
            found: schema_version,
            supported: PLUGIN_MANIFEST_SCHEMA_VERSION,
        });
    }

    Ok(())
}

fn validate_isolation(isolation: &PluginIsolation) -> Result<(), PluginManifestError> {
    let entrypoint = isolation
        .entrypoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match isolation.mode {
        PluginIsolationMode::Process | PluginIsolationMode::Wasm => {
            if entrypoint.is_none() {
                return Err(PluginManifestError::InvalidIsolation(
                    "process and wasm plugins require an entrypoint".to_string(),
                ));
            }
        }
        PluginIsolationMode::Builtin => {
            if entrypoint.is_some() {
                return Err(PluginManifestError::InvalidIsolation(
                    "built-in plugins must not declare an external entrypoint".to_string(),
                ));
            }
        }
    }

    Ok(())
}

fn validate_capabilities(
    capabilities: &[PluginCapabilityManifest],
) -> Result<(), PluginManifestError> {
    let mut seen = BTreeSet::new();

    for capability in capabilities {
        validate_non_empty("capability name", &capability.name)?;

        let key = format!("{}:{}", capability.kind.as_str(), capability.name);
        if !seen.insert(key.clone()) {
            return Err(PluginManifestError::DuplicateContribution(key));
        }

        for permission in &capability.permissions {
            validate_non_empty("capability permission resource", &permission.resource)?;
            validate_non_empty("capability permission action", &permission.action)?;
        }
    }

    Ok(())
}

fn validate_tools(tools: &[PluginToolManifest]) -> Result<(), PluginManifestError> {
    let mut seen = BTreeSet::new();

    for tool in tools {
        validate_non_empty("tool description", &tool.description)?;

        if !seen.insert(tool.name.clone()) {
            return Err(PluginManifestError::DuplicateContribution(format!(
                "tool {}",
                tool.name
            )));
        }
    }

    Ok(())
}

fn validate_hooks(hooks: &[PluginHookManifest]) -> Result<(), PluginManifestError> {
    let mut seen = BTreeSet::new();

    for hook in hooks {
        validate_non_empty("hook name", &hook.name)?;
        validate_non_empty("hook event", &hook.event)?;

        if !seen.insert(hook.name.clone()) {
            return Err(PluginManifestError::DuplicateContribution(format!(
                "hook {}",
                hook.name
            )));
        }
    }

    Ok(())
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), PluginManifestError> {
    if value.trim().is_empty() {
        return Err(PluginManifestError::InvalidManifest(format!(
            "{label} cannot be empty"
        )));
    }

    Ok(())
}

fn classify_parse_error(message: String) -> PluginManifestError {
    if message.contains("invalid plugin name") {
        PluginManifestError::InvalidName(message)
    } else {
        PluginManifestError::Parse(message)
    }
}
