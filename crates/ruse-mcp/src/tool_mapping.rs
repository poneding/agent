use ruse_tool::{ToolDefinition, ToolName};
use serde::{Deserialize, Serialize};

use crate::{McpError, McpServerId, McpToolDescriptor};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpLocalToolName {
    pub server_id: McpServerId,
    pub remote_name: String,
    pub local_name: ToolName,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub descriptor: McpToolDescriptor,
    pub local_name: ToolName,
    pub definition: ToolDefinition,
}

pub fn local_tool_name_for_mcp_tool(
    server_id: &McpServerId,
    remote_name: &str,
) -> Result<ToolName, McpError> {
    let normalized_remote = normalize_remote_tool_name(remote_name)?;
    let local_name = format!("mcp.{}.{}", server_id.as_str(), normalized_remote);
    ToolName::new(local_name).map_err(|err| McpError::InvalidToolDescriptor(err.to_string()))
}

pub fn mcp_descriptor_to_tool_definition(
    descriptor: &McpToolDescriptor,
) -> Result<McpToolDefinition, McpError> {
    let local_name = local_tool_name_for_mcp_tool(&descriptor.server_id, &descriptor.name)?;
    let definition = ToolDefinition {
        name: local_name.clone(),
        description: descriptor.description.clone(),
        input_schema: descriptor.input_schema.clone(),
    };

    Ok(McpToolDefinition {
        descriptor: descriptor.clone(),
        local_name,
        definition,
    })
}

fn normalize_remote_tool_name(remote_name: &str) -> Result<String, McpError> {
    let remote_name = remote_name.trim();
    if remote_name.is_empty() {
        return Err(McpError::InvalidToolDescriptor(
            "remote tool name cannot be empty".to_string(),
        ));
    }

    let mut has_alphanumeric = false;
    let normalized: String = remote_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                has_alphanumeric = true;
                ch.to_ascii_lowercase()
            } else if matches!(ch, '_' | '-' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if !has_alphanumeric {
        return Err(McpError::InvalidToolDescriptor(format!(
            "remote tool name `{remote_name}` must contain an ASCII letter or digit"
        )));
    }

    Ok(normalized)
}
