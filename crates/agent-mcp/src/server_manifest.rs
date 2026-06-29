use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::McpError;

pub const MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION: u32 = 1;

const MAX_MANIFEST_TEXT_CHARS: usize = 256;
const REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerExportManifest {
    pub schema_version: u32,
    #[serde(default)]
    pub profiles: Vec<McpServerExportProfileConfig>,
}

impl McpServerExportManifest {
    pub fn validate(self) -> Result<Self, McpError> {
        if self.schema_version != MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION {
            return Err(McpError::InvalidServerRequest(format!(
                "unsupported MCP server export manifest schema_version: {}",
                self.schema_version
            )));
        }
        if self.profiles.is_empty() {
            return Err(McpError::InvalidServerRequest(
                "MCP server export manifest requires at least one profile".to_string(),
            ));
        }

        let mut profile_names = BTreeSet::new();
        for profile in &self.profiles {
            let name = validate_identifier_text("profile name", &profile.name)?;
            if !profile_names.insert(name.clone()) {
                return Err(McpError::InvalidServerRequest(format!(
                    "duplicate MCP server export profile: {}",
                    redact_text(&name)
                )));
            }
            validate_profile(profile)?;
        }

        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerExportProfileConfig {
    pub name: String,
    #[serde(default)]
    pub tools: Vec<McpServerToolExportConfig>,
    #[serde(default)]
    pub resources: Vec<McpServerResourceExportConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerToolExportConfig {
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerResourceExportConfig {
    pub uri: String,
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub sensitive: bool,
}

pub fn parse_mcp_server_export_manifest_json(
    input: &str,
) -> Result<McpServerExportManifest, McpError> {
    serde_json::from_str::<McpServerExportManifest>(input)
        .map_err(|_| {
            McpError::InvalidServerRequest("invalid MCP server export manifest JSON".to_string())
        })?
        .validate()
}

pub fn select_mcp_server_export_profile<'a>(
    manifest: &'a McpServerExportManifest,
    profile_name: Option<&str>,
) -> Result<&'a McpServerExportProfileConfig, McpError> {
    if let Some(profile_name) = profile_name {
        let wanted = validate_identifier_text("profile name", profile_name)?;
        return manifest
            .profiles
            .iter()
            .find(|profile| profile.name == wanted)
            .ok_or_else(|| {
                McpError::InvalidServerRequest(format!(
                    "MCP server export profile not found: {}",
                    redact_text(&wanted)
                ))
            });
    }

    if let Some(default) = manifest
        .profiles
        .iter()
        .find(|profile| profile.name == "default")
    {
        return Ok(default);
    }

    if manifest.profiles.len() == 1 {
        return Ok(&manifest.profiles[0]);
    }

    Err(McpError::InvalidServerRequest(
        "MCP server export profile name is required when manifest has multiple profiles"
            .to_string(),
    ))
}

fn validate_profile(profile: &McpServerExportProfileConfig) -> Result<(), McpError> {
    let mut tool_names = BTreeSet::new();
    for tool in &profile.tools {
        let name = validate_identifier_text("tool name", &tool.name)?;
        if !tool_names.insert(name.clone()) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP server export tool in profile {}: {}",
                redact_text(&profile.name),
                redact_text(&name)
            )));
        }
    }

    let mut resource_uris = BTreeSet::new();
    for resource in &profile.resources {
        let uri = validate_resource_uri(&resource.uri)?;
        if !resource_uris.insert(uri.clone()) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP server export resource in profile {}: {}",
                redact_text(&profile.name),
                redact_text(&uri)
            )));
        }
        validate_public_text("resource name", &resource.name)?;
        validate_path_text("resource path", &resource.path)?;
        if let Some(mime_type) = &resource.mime_type {
            validate_public_text("resource mime_type", mime_type)?;
        }
    }

    Ok(())
}

fn validate_identifier_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP server export {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_MANIFEST_TEXT_CHARS))
}

fn validate_public_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP server export {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_MANIFEST_TEXT_CHARS))
}

fn validate_path_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP server export {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_MANIFEST_TEXT_CHARS))
}

fn validate_resource_uri(value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP server export resource uri: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_MANIFEST_TEXT_CHARS))
}

fn default_enabled() -> bool {
    true
}

fn redact_text(value: &str) -> String {
    if value.is_empty() {
        return "unknown".to_string();
    }
    if is_secret_like(value) {
        return REDACTED.to_string();
    }
    bound_text(value, MAX_MANIFEST_TEXT_CHARS)
}

fn bound_text(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let bounded = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}

fn is_secret_like(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace(['-', '.'], "_");
    normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("password")
        || normalized.contains("credential")
        || normalized.contains("private_key")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("sk_")
}
