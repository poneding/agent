use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::McpError;

const MAX_PROMPT_TEXT_CHARS: usize = 1024;
const MAX_PROMPT_METADATA_CHARS: usize = 256;
const REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPromptDescriptor {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl McpPromptDescriptor {
    pub fn new(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Result<Self, McpError> {
        let description = description
            .map(Into::into)
            .map(|description| validate_public_text("MCP prompt description", &description))
            .transpose()?;
        Ok(Self {
            name: validate_identifier_text("MCP prompt name", &name.into())?,
            description,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPromptMessage {
    pub role: String,
    pub content: McpPromptContent,
}

impl McpPromptMessage {
    pub fn text(role: impl Into<String>, text: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self {
            role: validate_identifier_text("MCP prompt message role", &role.into())?,
            content: McpPromptContent::text(text),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPromptContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl McpPromptContent {
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content_type: "text".to_string(),
            text: redact_text(&text.into(), MAX_PROMPT_TEXT_CHARS),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPromptListResult {
    pub prompts: Vec<McpPromptDescriptor>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPromptGetResult {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub messages: Vec<McpPromptMessage>,
}

impl McpPromptGetResult {
    pub fn new(
        descriptor: McpPromptDescriptor,
        messages: Vec<McpPromptMessage>,
    ) -> Result<Self, McpError> {
        if messages.is_empty() {
            return Err(McpError::InvalidServerRequest(
                "MCP prompt get result requires at least one message".to_string(),
            ));
        }
        Ok(Self {
            name: descriptor.name,
            description: descriptor.description,
            messages,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPromptExport {
    descriptor: McpPromptDescriptor,
    messages: Vec<McpPromptMessage>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct McpPromptExportRegistry {
    prompts: BTreeMap<String, McpPromptExport>,
}

impl McpPromptExportRegistry {
    pub fn export_prompt(&mut self, export: McpPromptExport) -> Result<(), McpError> {
        let name = export.descriptor.name.clone();
        if self.prompts.contains_key(&name) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP prompt export: {}",
                redact_text(&name, MAX_PROMPT_METADATA_CHARS)
            )));
        }
        self.prompts.insert(name, export);
        Ok(())
    }

    pub fn export_disabled_prompt(&mut self, mut export: McpPromptExport) -> Result<(), McpError> {
        export.enabled = false;
        self.export_prompt(export)
    }

    #[must_use]
    pub fn list_prompts(&self) -> McpPromptListResult {
        McpPromptListResult {
            prompts: self
                .prompts
                .values()
                .filter(|export| export.enabled)
                .map(|export| export.descriptor.clone())
                .collect(),
        }
    }

    pub fn get_prompt(&self, name: &str) -> Result<McpPromptGetResult, McpError> {
        self.prompts
            .get(name)
            .filter(|export| export.enabled)
            .map(McpPromptExport::get_result)
            .ok_or_else(|| {
                McpError::InvalidServerRequest(format!(
                    "MCP prompt export not found: {}",
                    redact_text(name, MAX_PROMPT_METADATA_CHARS)
                ))
            })
    }
}

impl McpPromptExport {
    pub fn new(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        messages: Vec<McpPromptMessage>,
        enabled: bool,
    ) -> Result<Self, McpError> {
        if messages.is_empty() {
            return Err(McpError::InvalidServerRequest(
                "MCP prompt export requires at least one messages entry".to_string(),
            ));
        }
        Ok(Self {
            descriptor: McpPromptDescriptor::new(name, description)?,
            messages,
            enabled,
        })
    }

    #[must_use]
    pub const fn descriptor(&self) -> &McpPromptDescriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn get_result(&self) -> McpPromptGetResult {
        McpPromptGetResult {
            name: self.descriptor.name.clone(),
            description: self.descriptor.description.clone(),
            messages: self.messages.clone(),
        }
    }
}

fn validate_identifier_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_text(trimmed, MAX_PROMPT_METADATA_CHARS)
        )));
    }
    Ok(bound_text(trimmed, MAX_PROMPT_METADATA_CHARS))
}

fn validate_public_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_text(trimmed, MAX_PROMPT_METADATA_CHARS)
        )));
    }
    Ok(bound_text(trimmed, MAX_PROMPT_METADATA_CHARS))
}

fn redact_text(value: &str, max_chars: usize) -> String {
    if value.is_empty() {
        return "unknown".to_string();
    }
    if is_secret_like(value) {
        return REDACTED.to_string();
    }
    bound_text(value, max_chars)
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
