use serde::{Deserialize, Serialize};

use crate::McpError;

pub const MCP_ROOTS_LIST_CHANGED_NOTIFICATION_METHOD: &str = "notifications/roots/list_changed";

const MAX_ROOT_TEXT_CHARS: usize = 256;
const REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpRoot {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl McpRoot {
    pub fn new<N>(uri: impl Into<String>, name: Option<N>) -> Result<Self, McpError>
    where
        N: Into<String>,
    {
        Ok(Self {
            uri: validate_uri_text("MCP root URI", &uri.into())?,
            name: name
                .map(|name| validate_optional_name_text("MCP root name", &name.into()))
                .transpose()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpRootListResult {
    pub roots: Vec<McpRoot>,
}

impl McpRootListResult {
    #[must_use]
    pub fn from_roots<I>(roots: I) -> Self
    where
        I: IntoIterator<Item = McpRoot>,
    {
        let mut roots = roots.into_iter().collect::<Vec<_>>();
        roots.sort_by(|left, right| {
            left.uri
                .cmp(&right.uri)
                .then_with(|| left.name.cmp(&right.name))
        });
        Self { roots }
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self { roots: Vec::new() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientRootsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

impl McpClientRootsCapability {
    #[must_use]
    pub const fn new(list_changed: bool) -> Self {
        Self { list_changed }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<McpClientRootsCapability>,
}

impl McpClientCapabilities {
    #[must_use]
    pub const fn with_roots(roots: McpClientRootsCapability) -> Self {
        Self { roots: Some(roots) }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpRootListChangedNotification {
    pub jsonrpc: String,
    pub method: String,
}

impl McpRootListChangedNotification {
    #[must_use]
    pub fn new() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: MCP_ROOTS_LIST_CHANGED_NOTIFICATION_METHOD.to_string(),
        }
    }
}

impl Default for McpRootListChangedNotification {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_uri_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return Err(McpError::InvalidConfig(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed))
}

fn validate_optional_name_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidConfig(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed))
}

fn redact_text(value: &str) -> String {
    if value.is_empty() {
        return "unknown".to_string();
    }
    if is_secret_like(value) {
        return REDACTED.to_string();
    }
    bound_text(value)
}

fn bound_text(value: &str) -> String {
    let mut chars = value.chars();
    let bounded = chars.by_ref().take(MAX_ROOT_TEXT_CHARS).collect::<String>();
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
