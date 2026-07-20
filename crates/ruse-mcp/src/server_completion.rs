use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::McpError;

const MAX_COMPLETION_TEXT_CHARS: usize = 256;
const REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpCompletionRequest {
    #[serde(rename = "ref")]
    pub completion_ref: McpCompletionRef,
    pub argument: McpCompletionArgument,
}

impl McpCompletionRequest {
    #[must_use]
    pub const fn new(completion_ref: McpCompletionRef, argument: McpCompletionArgument) -> Self {
        Self {
            completion_ref,
            argument,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpCompletionRef {
    #[serde(rename = "ref/prompt")]
    Prompt { name: String },
    #[serde(rename = "ref/resource")]
    Resource { uri: String },
}

impl McpCompletionRef {
    pub fn prompt(name: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self::Prompt {
            name: validate_identifier_text("MCP completion prompt ref", &name.into())?,
        })
    }

    pub fn resource(uri: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self::Resource {
            uri: validate_uri_text("MCP completion resource ref", &uri.into())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpCompletionArgument {
    pub name: String,
    pub value: String,
}

impl McpCompletionArgument {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self {
            name: validate_identifier_text("MCP completion argument name", &name.into())?,
            value: validate_prefix_text("MCP completion argument value", &value.into())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpCompletionResult {
    pub completion: McpCompletionValues,
}

impl McpCompletionResult {
    pub fn from_values<I, V>(prefix: &str, values: I, max_values: usize) -> Result<Self, McpError>
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        let prefix = validate_prefix_text("MCP completion prefix", prefix)?;
        let mut all_matches = BTreeSet::new();
        for value in values {
            let value = validate_public_text("MCP completion value", value.as_ref())?;
            if value.starts_with(&prefix) {
                all_matches.insert(value);
            }
        }

        let total = all_matches.len();
        let selected = all_matches.into_iter().take(max_values).collect::<Vec<_>>();
        Ok(Self {
            completion: McpCompletionValues {
                has_more: total > selected.len(),
                values: selected,
                total,
            },
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpCompletionValues {
    pub values: Vec<String>,
    pub total: usize,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

fn validate_identifier_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed))
}

fn validate_uri_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed))
}

fn validate_prefix_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed))
}

fn validate_public_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
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
    let bounded = chars
        .by_ref()
        .take(MAX_COMPLETION_TEXT_CHARS)
        .collect::<String>();
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
