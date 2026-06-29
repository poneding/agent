use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{McpError, McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcSuccess};

const MAX_CLIENT_HTTP_BODY_BYTES: usize = 256 * 1024;
const MAX_CLIENT_HTTP_MESSAGE_CHARS: usize = 256;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientHttpResponseFrame {
    Success(McpJsonRpcSuccess),
    Failure(McpJsonRpcFailure),
}

impl McpClientHttpResponseFrame {
    #[must_use]
    pub const fn id(&self) -> &McpJsonRpcId {
        match self {
            Self::Success(success) => &success.id,
            Self::Failure(failure) => &failure.id,
        }
    }
}

pub fn render_mcp_client_http_request_body(
    id: McpJsonRpcId,
    method: impl Into<String>,
    params: Value,
) -> Result<Value, McpError> {
    if matches!(id, McpJsonRpcId::Null) {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP request id cannot be null".to_string(),
        ));
    }

    let method = method.into();
    if method.trim().is_empty() {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP method cannot be empty".to_string(),
        ));
    }

    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    }))
}

pub fn parse_mcp_client_http_response_text(
    body: &str,
) -> Result<McpClientHttpResponseFrame, McpError> {
    if body.len() > MAX_CLIENT_HTTP_BODY_BYTES {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP response body too large".to_string(),
        ));
    }

    let value = serde_json::from_str::<Value>(body).map_err(|_| {
        McpError::InvalidServerRequest("invalid MCP client HTTP response body".to_string())
    })?;
    parse_mcp_client_http_response_body(&value)
}

pub fn parse_mcp_client_http_response_body(
    body: &Value,
) -> Result<McpClientHttpResponseFrame, McpError> {
    if body.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP response JSON-RPC version must be 2.0".to_string(),
        ));
    }
    if body.get("id").is_none() {
        return Err(McpError::InvalidServerRequest(
            "MCP client HTTP response id missing".to_string(),
        ));
    }

    match (body.get("result").is_some(), body.get("error").is_some()) {
        (true, false) => serde_json::from_value::<McpJsonRpcSuccess>(body.clone())
            .map(McpClientHttpResponseFrame::Success)
            .map_err(|_| {
                McpError::InvalidServerRequest(
                    "invalid MCP client HTTP success response".to_string(),
                )
            }),
        (false, true) => {
            let mut failure =
                serde_json::from_value::<McpJsonRpcFailure>(body.clone()).map_err(|_| {
                    McpError::InvalidServerRequest(
                        "invalid MCP client HTTP failure response".to_string(),
                    )
                })?;
            failure.error.message = public_http_message(&failure.error.message);
            Ok(McpClientHttpResponseFrame::Failure(failure))
        }
        _ => Err(McpError::InvalidServerRequest(
            "MCP client HTTP response must contain exactly one result or error".to_string(),
        )),
    }
}

pub fn validate_mcp_client_http_status(status: u16) -> Result<(), McpError> {
    if (200..300).contains(&status) {
        return Ok(());
    }

    Err(McpError::Client(format!(
        "MCP client HTTP request failed with HTTP status {status}"
    )))
}

fn public_http_message(value: &str) -> String {
    truncate_chars(
        &redact_secret_like_text(value),
        MAX_CLIENT_HTTP_MESSAGE_CHARS,
    )
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return value.chars().take(max_chars).collect();
    }

    let prefix = value.chars().take(max_chars - 3).collect::<String>();
    format!("{prefix}...")
}

fn redact_secret_like_text(value: &str) -> String {
    if is_secret_like(value) {
        "[redacted]".to_string()
    } else {
        value.to_string()
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
