use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use agent_policy::{CapabilityAction, PermissionRequest, ResourceKind, ResourceScope};
use agent_tool::ToolName;
use agent_types::{ActorRef, CorrelationId};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::PluginManifestError;
use crate::manifest::{PluginCapabilityManifest, PluginManifest, PluginName};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapabilityKind {
    Tool,
    Hook,
    Filesystem,
    Command,
    Network,
    Mcp,
    Secret,
}

impl PluginCapabilityKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::Hook => "hook",
            Self::Filesystem => "filesystem",
            Self::Command => "command",
            Self::Network => "network",
            Self::Mcp => "mcp",
            Self::Secret => "secret",
        }
    }
}

impl fmt::Display for PluginCapabilityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PluginPermission {
    pub resource: String,
    pub action: String,
}

impl PluginPermission {
    pub fn new(
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<Self, PluginManifestError> {
        let resource = resource.into();
        let action = action.into();
        validate_permission_part("permission resource", &resource)?;
        validate_permission_part("permission action", &action)?;
        Ok(Self { resource, action })
    }
}

impl fmt::Display for PluginPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.resource, self.action)
    }
}

impl FromStr for PluginPermission {
    type Err = PluginManifestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (resource, action) = value.split_once('.').ok_or_else(|| {
            PluginManifestError::InvalidManifest(format!(
                "plugin permission must use resource.action form: {value}"
            ))
        })?;
        Self::new(resource, action)
    }
}

impl Serialize for PluginPermission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PluginPermission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginCapabilitySet {
    pub plugin_name: PluginName,
    pub capabilities: Vec<PluginCapabilityManifest>,
}

pub fn validate_plugin_capabilities(
    manifest: &PluginManifest,
) -> Result<PluginCapabilitySet, PluginManifestError> {
    validate_duplicate_capabilities(&manifest.capabilities)?;
    validate_permission_parts(&manifest.capabilities)?;
    validate_tool_capabilities(manifest)?;
    validate_hook_capabilities(manifest)?;

    Ok(PluginCapabilitySet {
        plugin_name: manifest.name.clone(),
        capabilities: manifest.capabilities.clone(),
    })
}

pub fn permission_request_for_plugin_tool(
    plugin_name: &PluginName,
    tool_name: &ToolName,
    actor: ActorRef,
    correlation_id: CorrelationId,
) -> Result<PermissionRequest, PluginManifestError> {
    let scope = ResourceScope::new(ResourceKind::Tool, tool_name.as_str())
        .map_err(|error| PluginManifestError::InvalidManifest(error.to_string()))?;
    PermissionRequest::new(
        actor,
        CapabilityAction::Read,
        scope,
        format!("plugin tool execution requested: {}", plugin_name.as_str()),
        correlation_id,
    )
    .map_err(|error| PluginManifestError::InvalidManifest(error.to_string()))
}

fn validate_duplicate_capabilities(
    capabilities: &[PluginCapabilityManifest],
) -> Result<(), PluginManifestError> {
    let mut seen = BTreeSet::new();

    for capability in capabilities {
        let key = (capability.kind, capability.name.clone());
        if !seen.insert(key) {
            return Err(PluginManifestError::DuplicateContribution(format!(
                "{}:{}",
                capability.kind, capability.name
            )));
        }
    }

    Ok(())
}

fn validate_permission_parts(
    capabilities: &[PluginCapabilityManifest],
) -> Result<(), PluginManifestError> {
    for capability in capabilities {
        for permission in &capability.permissions {
            validate_permission_part("permission resource", &permission.resource)?;
            validate_permission_part("permission action", &permission.action)?;
        }
    }

    Ok(())
}

fn validate_tool_capabilities(manifest: &PluginManifest) -> Result<(), PluginManifestError> {
    for tool in &manifest.tools {
        let has_capability = manifest.capabilities.iter().any(|capability| {
            capability.kind == PluginCapabilityKind::Tool && capability.name == tool.name.as_str()
        });

        if !has_capability {
            return Err(PluginManifestError::InvalidManifest(format!(
                "tool {} is missing a matching tool capability",
                tool.name
            )));
        }
    }

    Ok(())
}

fn validate_hook_capabilities(manifest: &PluginManifest) -> Result<(), PluginManifestError> {
    for hook in &manifest.hooks {
        let has_capability = manifest.capabilities.iter().any(|capability| {
            capability.kind == PluginCapabilityKind::Hook && capability.name == hook.name
        });

        if !has_capability {
            return Err(PluginManifestError::InvalidManifest(format!(
                "hook {} is missing a matching hook capability",
                hook.name
            )));
        }
    }

    Ok(())
}

fn validate_permission_part(label: &str, value: &str) -> Result<(), PluginManifestError> {
    if value.trim().is_empty() {
        return Err(PluginManifestError::InvalidManifest(format!(
            "{label} cannot be empty"
        )));
    }

    Ok(())
}
