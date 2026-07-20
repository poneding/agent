use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{McpError, McpServerCapabilities};

pub const MCP_SERVER_PROTOCOL_VERSION: &str = "2025-11-25";
const MCP_SERVER_MAX_MESSAGE_CHARS: usize = 256;
const MCP_SERVER_REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    pub protocol_version: String,
}

impl McpServerInfo {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        protocol_version: impl Into<String>,
    ) -> Result<Self, McpError> {
        Ok(Self {
            name: validate_public_server_text("name", &name.into())?,
            version: validate_public_server_text("version", &version.into())?,
            protocol_version: validate_public_server_text(
                "protocol_version",
                &protocol_version.into(),
            )?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerState {
    pub info: McpServerInfo,
    pub capabilities: McpServerCapabilities,
    pub initialized: bool,
}

impl McpServerState {
    #[must_use]
    pub const fn new(info: McpServerInfo, capabilities: McpServerCapabilities) -> Self {
        Self {
            info,
            capabilities,
            initialized: false,
        }
    }

    #[must_use]
    pub const fn initialized(mut self) -> Self {
        self.initialized = true;
        self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum McpServerMethod {
    Initialize,
    PromptsList,
    PromptsGet,
    CompletionComplete,
    ToolsList,
    ToolsCall,
    ResourcesList,
    ResourcesTemplatesList,
    ResourcesRead,
}

impl McpServerMethod {
    pub fn parse(value: &str) -> Result<Self, McpError> {
        match value {
            "initialize" => Ok(Self::Initialize),
            "prompts/list" => Ok(Self::PromptsList),
            "prompts/get" => Ok(Self::PromptsGet),
            "completion/complete" => Ok(Self::CompletionComplete),
            "tools/list" => Ok(Self::ToolsList),
            "tools/call" => Ok(Self::ToolsCall),
            "resources/list" => Ok(Self::ResourcesList),
            "resources/templates/list" => Ok(Self::ResourcesTemplatesList),
            "resources/read" => Ok(Self::ResourcesRead),
            other => Err(McpError::InvalidServerRequest(format!(
                "unsupported method {}",
                redact_server_text(other)
            ))),
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::PromptsList => "prompts/list",
            Self::PromptsGet => "prompts/get",
            Self::CompletionComplete => "completion/complete",
            Self::ToolsList => "tools/list",
            Self::ToolsCall => "tools/call",
            Self::ResourcesList => "resources/list",
            Self::ResourcesTemplatesList => "resources/templates/list",
            Self::ResourcesRead => "resources/read",
        }
    }
}

impl Serialize for McpServerMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for McpServerMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpJsonRpcId {
    String(String),
    Number(i64),
    Null,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpJsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<McpJsonRpcId>,
    pub method: String,
    #[serde(default = "empty_params")]
    pub params: Value,
}

impl McpJsonRpcRequest {
    pub fn new(
        id: Option<McpJsonRpcId>,
        method: impl Into<String>,
        params: Value,
    ) -> Result<Self, McpError> {
        let method = method.into();
        McpServerMethod::parse(&method)?;
        Ok(Self {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpJsonRpcSuccess {
    pub jsonrpc: String,
    pub id: McpJsonRpcId,
    pub result: Value,
}

impl McpJsonRpcSuccess {
    #[must_use]
    pub fn new(id: McpJsonRpcId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpJsonRpcFailure {
    pub jsonrpc: String,
    pub id: McpJsonRpcId,
    pub error: McpJsonRpcError,
}

impl McpJsonRpcFailure {
    #[must_use]
    pub fn new(id: McpJsonRpcId, error: McpJsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            error,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpJsonRpcError {
    pub code: i32,
    pub message: String,
}

impl McpJsonRpcError {
    #[must_use]
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(-32700, message)
    }

    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(-32600, message)
    }

    #[must_use]
    pub fn method_not_found(method: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(
            -32601,
            format!("method not found: {} {}", method.into(), detail.into()),
        )
    }

    #[must_use]
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(-32602, message)
    }

    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(-32603, message)
    }

    fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: redact_server_text(&message.into()),
        }
    }
}

fn empty_params() -> Value {
    serde_json::json!({})
}

fn validate_public_server_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(McpError::InvalidServerRequest(format!(
            "{field} cannot be empty"
        )));
    }
    if is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "{field} cannot contain secret-like text"
        )));
    }
    Ok(bound_chars(trimmed, MCP_SERVER_MAX_MESSAGE_CHARS))
}

fn redact_server_text(value: &str) -> String {
    if is_secret_like(value) {
        return MCP_SERVER_REDACTED.to_string();
    }
    bound_chars(value, MCP_SERVER_MAX_MESSAGE_CHARS)
}

fn bound_chars(value: &str, max_chars: usize) -> String {
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
