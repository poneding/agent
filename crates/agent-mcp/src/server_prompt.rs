use std::collections::{BTreeMap, BTreeSet};

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<McpPromptArgument>,
}

impl McpPromptDescriptor {
    pub fn new(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Result<Self, McpError> {
        Self::new_with_arguments(name, description, Vec::new())
    }

    pub fn new_with_arguments(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        arguments: Vec<McpPromptArgument>,
    ) -> Result<Self, McpError> {
        let description = description
            .map(Into::into)
            .map(|description| validate_public_text("MCP prompt description", &description))
            .transpose()?;
        validate_prompt_arguments(&arguments)?;
        Ok(Self {
            name: validate_identifier_text("MCP prompt name", &name.into())?,
            description,
            arguments,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

impl McpPromptArgument {
    pub fn new(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        required: bool,
    ) -> Result<Self, McpError> {
        let description = description
            .map(Into::into)
            .map(|description| {
                validate_public_text("MCP prompt argument description", &description)
            })
            .transpose()?;
        Ok(Self {
            name: validate_identifier_text("MCP prompt argument name", &name.into())?,
            description,
            required,
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
pub struct McpPromptGetParams {
    pub name: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub arguments: BTreeMap<String, String>,
}

impl McpPromptGetParams {
    pub fn new(name: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self {
            name: validate_identifier_text("MCP prompt get name", &name.into())?,
            arguments: BTreeMap::new(),
        })
    }

    pub fn with_argument(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, McpError> {
        let name = validate_identifier_text("MCP prompt argument name", &name.into())?;
        let value = redact_text(&value.into(), MAX_PROMPT_TEXT_CHARS);
        self.arguments.insert(name, value);
        Ok(self)
    }
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

    pub fn get_prompt_with_arguments(
        &self,
        name: &str,
        arguments: &BTreeMap<String, String>,
    ) -> Result<McpPromptGetResult, McpError> {
        self.prompts
            .get(name)
            .filter(|export| export.enabled)
            .ok_or_else(|| {
                McpError::InvalidServerRequest(format!(
                    "MCP prompt export not found: {}",
                    redact_text(name, MAX_PROMPT_METADATA_CHARS)
                ))
            })?
            .get_result_with_arguments(arguments)
    }
}

impl McpPromptExport {
    pub fn new(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        messages: Vec<McpPromptMessage>,
        enabled: bool,
    ) -> Result<Self, McpError> {
        Self::new_with_arguments(name, description, Vec::new(), messages, enabled)
    }

    pub fn new_with_arguments(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        arguments: Vec<McpPromptArgument>,
        messages: Vec<McpPromptMessage>,
        enabled: bool,
    ) -> Result<Self, McpError> {
        if messages.is_empty() {
            return Err(McpError::InvalidServerRequest(
                "MCP prompt export requires at least one messages entry".to_string(),
            ));
        }
        let descriptor = McpPromptDescriptor::new_with_arguments(name, description, arguments)?;
        validate_prompt_message_templates(&descriptor.arguments, &messages)?;
        Ok(Self {
            descriptor,
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

    pub fn get_result_with_arguments(
        &self,
        arguments: &BTreeMap<String, String>,
    ) -> Result<McpPromptGetResult, McpError> {
        validate_prompt_argument_values(&self.descriptor.arguments, arguments)?;
        let messages = self
            .messages
            .iter()
            .map(|message| render_prompt_message(message, arguments))
            .collect::<Result<Vec<_>, _>>()?;

        McpPromptGetResult::new(self.descriptor.clone(), messages)
    }
}

fn validate_prompt_arguments(arguments: &[McpPromptArgument]) -> Result<(), McpError> {
    let mut names = BTreeSet::new();
    for argument in arguments {
        if !names.insert(argument.name.clone()) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP prompt argument: {}",
                redact_text(&argument.name, MAX_PROMPT_METADATA_CHARS)
            )));
        }
    }
    Ok(())
}

fn validate_prompt_argument_values(
    declared_arguments: &[McpPromptArgument],
    arguments: &BTreeMap<String, String>,
) -> Result<(), McpError> {
    let declared_names = declared_arguments
        .iter()
        .map(|argument| argument.name.as_str())
        .collect::<BTreeSet<_>>();

    for name in arguments.keys() {
        let name = validate_identifier_text("MCP prompt argument name", name)?;
        if !declared_names.contains(name.as_str()) {
            return Err(McpError::InvalidServerRequest(format!(
                "unknown MCP prompt argument: {}",
                redact_text(&name, MAX_PROMPT_METADATA_CHARS)
            )));
        }
    }

    for argument in declared_arguments {
        if argument.required {
            match arguments.get(&argument.name) {
                Some(value) if !value.trim().is_empty() => {}
                _ => {
                    return Err(McpError::InvalidServerRequest(format!(
                        "missing required MCP prompt argument: {}",
                        redact_text(&argument.name, MAX_PROMPT_METADATA_CHARS)
                    )));
                }
            }
        }
    }

    Ok(())
}

fn validate_prompt_message_templates(
    declared_arguments: &[McpPromptArgument],
    messages: &[McpPromptMessage],
) -> Result<(), McpError> {
    let declared_names = declared_arguments
        .iter()
        .map(|argument| argument.name.as_str())
        .collect::<BTreeSet<_>>();

    for message in messages {
        for reference in template_references(&message.content.text)? {
            if !declared_names.contains(reference.as_str()) {
                return Err(McpError::InvalidServerRequest(format!(
                    "undeclared MCP prompt argument in template: {}",
                    redact_text(&reference, MAX_PROMPT_METADATA_CHARS)
                )));
            }
        }
    }

    Ok(())
}

fn render_prompt_message(
    message: &McpPromptMessage,
    arguments: &BTreeMap<String, String>,
) -> Result<McpPromptMessage, McpError> {
    Ok(McpPromptMessage {
        role: message.role.clone(),
        content: McpPromptContent::text(render_prompt_text(&message.content.text, arguments)?),
    })
}

fn render_prompt_text(
    template: &str,
    arguments: &BTreeMap<String, String>,
) -> Result<String, McpError> {
    let mut rendered = String::new();
    let mut cursor = 0;

    while let Some(start_relative) = template[cursor..].find("{{") {
        let start = cursor + start_relative;
        rendered.push_str(&template[cursor..start]);
        let reference_start = start + 2;
        let Some(end_relative) = template[reference_start..].find("}}") else {
            return Err(McpError::InvalidServerRequest(
                "unclosed MCP prompt template reference".to_string(),
            ));
        };
        let end = reference_start + end_relative;
        let reference = validate_identifier_text(
            "MCP prompt template reference",
            template[reference_start..end].trim(),
        )?;
        let value = arguments
            .get(&reference)
            .map(|value| redact_text(value, MAX_PROMPT_TEXT_CHARS))
            .unwrap_or_default();
        rendered.push_str(&value);
        cursor = end + 2;
    }

    rendered.push_str(&template[cursor..]);
    Ok(rendered)
}

fn template_references(template: &str) -> Result<Vec<String>, McpError> {
    let mut references = Vec::new();
    let mut cursor = 0;

    while let Some(start_relative) = template[cursor..].find("{{") {
        let start = cursor + start_relative;
        let reference_start = start + 2;
        let Some(end_relative) = template[reference_start..].find("}}") else {
            return Err(McpError::InvalidServerRequest(
                "unclosed MCP prompt template reference".to_string(),
            ));
        };
        let end = reference_start + end_relative;
        references.push(validate_identifier_text(
            "MCP prompt template reference",
            template[reference_start..end].trim(),
        )?);
        cursor = end + 2;
    }

    Ok(references)
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
