use serde::{Deserialize, Serialize};

use crate::{McpError, McpServerId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub server_id: McpServerId,
    pub name: String,
    pub title: Option<String>,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl McpToolDescriptor {
    pub fn new<T>(
        server_id: McpServerId,
        name: impl Into<String>,
        title: Option<T>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Result<Self, McpError>
    where
        T: Into<String>,
    {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(McpError::InvalidToolDescriptor(
                "tool name cannot be empty".to_string(),
            ));
        }

        let description = description.into();
        if description.trim().is_empty() {
            return Err(McpError::InvalidToolDescriptor(
                "tool description cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            server_id,
            name,
            title: title.map(Into::into),
            description,
            input_schema,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpToolListResult {
    pub server_id: McpServerId,
    pub tools: Vec<McpToolDescriptor>,
}

impl McpToolListResult {
    #[must_use]
    pub fn new(server_id: McpServerId, tools: Vec<McpToolDescriptor>) -> Self {
        Self { server_id, tools }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpToolCallRequest {
    pub server_id: McpServerId,
    pub remote_tool_name: String,
    pub input_json: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpToolCallResult {
    pub output_json: serde_json::Value,
}

impl McpToolCallResult {
    #[must_use]
    pub fn new(output_json: serde_json::Value) -> Self {
        Self { output_json }
    }
}
