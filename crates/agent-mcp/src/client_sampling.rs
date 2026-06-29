use std::collections::BTreeMap;

use serde::de::Error as _;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::McpError;

const MAX_SAMPLING_PUBLIC_TEXT_CHARS: usize = 256;
const SAMPLING_REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct McpClientSamplingCapability {
    pub tools: bool,
    pub context: bool,
}

impl McpClientSamplingCapability {
    #[must_use]
    pub const fn basic() -> Self {
        Self {
            tools: false,
            context: false,
        }
    }

    #[must_use]
    pub const fn with_tools() -> Self {
        Self {
            tools: true,
            context: false,
        }
    }

    #[must_use]
    pub const fn with_context() -> Self {
        Self {
            tools: false,
            context: true,
        }
    }

    #[must_use]
    pub const fn with_tools_and_context() -> Self {
        Self {
            tools: true,
            context: true,
        }
    }
}

impl Serialize for McpClientSamplingCapability {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut len = 0;
        if self.tools {
            len += 1;
        }
        if self.context {
            len += 1;
        }

        let empty = BTreeMap::<String, Value>::new();
        let mut map = serializer.serialize_map(Some(len))?;
        if self.tools {
            map.serialize_entry("tools", &empty)?;
        }
        if self.context {
            map.serialize_entry("context", &empty)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for McpClientSamplingCapability {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = BTreeMap::<String, Value>::deserialize(deserializer)?;
        Ok(Self {
            tools: value.contains_key("tools"),
            context: value.contains_key("context"),
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpSamplingRole {
    User,
    Assistant,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum McpSamplingIncludeContext {
    #[default]
    None,
    ThisServer,
    AllServers,
}

impl McpSamplingIncludeContext {
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct McpSamplingContentList {
    pub content: Vec<McpSamplingContent>,
}

impl McpSamplingContentList {
    #[must_use]
    pub fn from_one(content: McpSamplingContent) -> Self {
        Self {
            content: vec![content],
        }
    }

    #[must_use]
    pub fn from_many<I>(content: I) -> Self
    where
        I: IntoIterator<Item = McpSamplingContent>,
    {
        Self {
            content: content.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

impl From<McpSamplingContent> for McpSamplingContentList {
    fn from(content: McpSamplingContent) -> Self {
        Self::from_one(content)
    }
}

impl Serialize for McpSamplingContentList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.content.len() == 1 {
            self.content[0].serialize(serializer)
        } else {
            self.content.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for McpSamplingContentList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Value::Array(_) = value {
            Ok(Self {
                content: serde_json::from_value(value).map_err(D::Error::custom)?,
            })
        } else {
            Ok(Self::from_one(
                serde_json::from_value(value).map_err(D::Error::custom)?,
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpSamplingContent {
    Text {
        text: String,
    },
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    Audio {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        #[serde(rename = "toolUseId")]
        tool_use_id: String,
        content: Vec<McpSamplingContent>,
    },
}

impl McpSamplingContent {
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self::Image {
            data: validate_non_empty_sampling_text("sampling image data", &data.into())?,
            mime_type: validate_public_sampling_text(
                "sampling image MIME type",
                &mime_type.into(),
            )?,
        })
    }

    pub fn audio(data: impl Into<String>, mime_type: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self::Audio {
            data: validate_non_empty_sampling_text("sampling audio data", &data.into())?,
            mime_type: validate_public_sampling_text(
                "sampling audio MIME type",
                &mime_type.into(),
            )?,
        })
    }

    pub fn tool_use(
        id: impl Into<String>,
        name: impl Into<String>,
        input: Value,
    ) -> Result<Self, McpError> {
        Ok(Self::ToolUse {
            id: validate_public_sampling_text("sampling tool use id", &id.into())?,
            name: validate_public_sampling_text("sampling tool name", &name.into())?,
            input,
        })
    }

    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: Vec<McpSamplingContent>,
    ) -> Result<Self, McpError> {
        Ok(Self::ToolResult {
            tool_use_id: validate_public_sampling_text(
                "sampling tool result id",
                &tool_use_id.into(),
            )?,
            content,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpSamplingMessage {
    pub role: McpSamplingRole,
    pub content: McpSamplingContentList,
}

impl McpSamplingMessage {
    #[must_use]
    pub fn new(role: McpSamplingRole, content: impl Into<McpSamplingContentList>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    #[must_use]
    pub fn user(content: impl Into<McpSamplingContentList>) -> Self {
        Self::new(McpSamplingRole::User, content)
    }

    #[must_use]
    pub fn assistant(content: impl Into<McpSamplingContentList>) -> Self {
        Self::new(McpSamplingRole::Assistant, content)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpModelHint {
    pub name: String,
}

impl McpModelHint {
    pub fn new(name: impl Into<String>) -> Result<Self, McpError> {
        Ok(Self {
            name: validate_public_sampling_text("sampling model hint", &name.into())?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct McpModelPreferences {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<McpModelHint>,
    #[serde(
        rename = "costPriority",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cost_priority: Option<f64>,
    #[serde(
        rename = "speedPriority",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub speed_priority: Option<f64>,
    #[serde(
        rename = "intelligencePriority",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub intelligence_priority: Option<f64>,
}

impl McpModelPreferences {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hints: Vec::new(),
            cost_priority: None,
            speed_priority: None,
            intelligence_priority: None,
        }
    }

    #[must_use]
    pub fn with_hint(mut self, hint: McpModelHint) -> Self {
        self.hints.push(hint);
        self
    }

    #[must_use]
    pub const fn with_cost_priority(mut self, priority: f64) -> Self {
        self.cost_priority = Some(priority);
        self
    }

    #[must_use]
    pub const fn with_speed_priority(mut self, priority: f64) -> Self {
        self.speed_priority = Some(priority);
        self
    }

    #[must_use]
    pub const fn with_intelligence_priority(mut self, priority: f64) -> Self {
        self.intelligence_priority = Some(priority);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpCreateMessageRequestParams {
    pub messages: Vec<McpSamplingMessage>,
    #[serde(
        rename = "modelPreferences",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub model_preferences: Option<McpModelPreferences>,
    #[serde(
        rename = "systemPrompt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub system_prompt: Option<String>,
    #[serde(
        rename = "includeContext",
        default,
        skip_serializing_if = "McpSamplingIncludeContext::is_none"
    )]
    pub include_context: McpSamplingIncludeContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(rename = "maxTokens")]
    pub max_tokens: u32,
    #[serde(
        rename = "stopSequences",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub stop_sequences: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Value>,
    #[serde(
        rename = "toolChoice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub tool_choice: Option<Value>,
}

impl McpCreateMessageRequestParams {
    #[must_use]
    pub fn new(messages: Vec<McpSamplingMessage>, max_tokens: u32) -> Self {
        Self {
            messages,
            model_preferences: None,
            system_prompt: None,
            include_context: McpSamplingIncludeContext::None,
            temperature: None,
            max_tokens,
            stop_sequences: Vec::new(),
            metadata: None,
            tools: Vec::new(),
            tool_choice: None,
        }
    }

    #[must_use]
    pub fn with_model_preferences(mut self, model_preferences: McpModelPreferences) -> Self {
        self.model_preferences = Some(model_preferences);
        self
    }

    #[must_use]
    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    #[must_use]
    pub const fn with_include_context(
        mut self,
        include_context: McpSamplingIncludeContext,
    ) -> Self {
        self.include_context = include_context;
        self
    }

    #[must_use]
    pub const fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    #[must_use]
    pub fn with_stop_sequence(mut self, stop_sequence: impl Into<String>) -> Self {
        self.stop_sequences.push(stop_sequence.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpCreateMessageResult {
    pub role: McpSamplingRole,
    pub content: McpSamplingContentList,
    pub model: String,
    #[serde(
        rename = "stopReason",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub stop_reason: Option<String>,
}

impl McpCreateMessageResult {
    pub fn new<S>(
        role: McpSamplingRole,
        content: impl Into<McpSamplingContentList>,
        model: impl Into<String>,
        stop_reason: Option<S>,
    ) -> Result<Self, McpError>
    where
        S: Into<String>,
    {
        Ok(Self {
            role,
            content: content.into(),
            model: validate_public_sampling_text("sampling model", &model.into())?,
            stop_reason: stop_reason
                .map(|value| validate_public_sampling_text("sampling stop reason", &value.into()))
                .transpose()?,
        })
    }
}

fn validate_non_empty_sampling_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: unknown"
        )));
    }
    Ok(bound_sampling_text(trimmed))
}

fn validate_public_sampling_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_sampling_text(trimmed)
        )));
    }
    Ok(bound_sampling_text(trimmed))
}

fn redact_sampling_text(value: &str) -> String {
    if value.is_empty() {
        return "unknown".to_string();
    }
    if is_secret_like(value) {
        return SAMPLING_REDACTED.to_string();
    }
    bound_sampling_text(value)
}

fn bound_sampling_text(value: &str) -> String {
    let mut chars = value.chars();
    let bounded = chars
        .by_ref()
        .take(MAX_SAMPLING_PUBLIC_TEXT_CHARS)
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
