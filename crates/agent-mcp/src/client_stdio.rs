use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{McpError, McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcSuccess};

const MAX_CLIENT_STDIO_FRAME_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientStdioResponseFrame {
    Success(McpJsonRpcSuccess),
    Failure(McpJsonRpcFailure),
}

pub fn render_mcp_client_stdio_request_frame(
    id: McpJsonRpcId,
    method: impl Into<String>,
    params: Value,
) -> Result<String, McpError> {
    if matches!(id, McpJsonRpcId::Null) {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio request id cannot be null".to_string(),
        ));
    }

    let method = method.into();
    if method.trim().is_empty() {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio method cannot be empty".to_string(),
        ));
    }

    let request = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let mut rendered = serde_json::to_string(&request).map_err(|_| {
        McpError::InvalidServerRequest("failed to serialize MCP client stdio request".to_string())
    })?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn parse_mcp_client_stdio_response_frame(
    frame: &str,
) -> Result<McpClientStdioResponseFrame, McpError> {
    if frame.len() > MAX_CLIENT_STDIO_FRAME_BYTES {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio response frame too large".to_string(),
        ));
    }

    let value = serde_json::from_str::<Value>(frame.trim_end()).map_err(|_| {
        McpError::InvalidServerRequest("invalid MCP client stdio response frame".to_string())
    })?;
    if value.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio response JSON-RPC version must be 2.0".to_string(),
        ));
    }
    if value.get("id").is_none() {
        return Err(McpError::InvalidServerRequest(
            "MCP client stdio response id missing".to_string(),
        ));
    }

    match (value.get("result").is_some(), value.get("error").is_some()) {
        (true, false) => serde_json::from_value::<McpJsonRpcSuccess>(value)
            .map(McpClientStdioResponseFrame::Success)
            .map_err(|_| {
                McpError::InvalidServerRequest(
                    "invalid MCP client stdio success response".to_string(),
                )
            }),
        (false, true) => serde_json::from_value::<McpJsonRpcFailure>(value)
            .map(McpClientStdioResponseFrame::Failure)
            .map_err(|_| {
                McpError::InvalidServerRequest(
                    "invalid MCP client stdio failure response".to_string(),
                )
            }),
        _ => Err(McpError::InvalidServerRequest(
            "MCP client stdio response must contain exactly one result or error".to_string(),
        )),
    }
}
