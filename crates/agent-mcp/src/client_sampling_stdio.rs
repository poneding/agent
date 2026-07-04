use crate::{
    McpCreateMessageRequestParams, McpError, McpJsonRpcError, McpJsonRpcFailure, McpJsonRpcId,
    McpJsonRpcRequest, McpJsonRpcSuccess, McpSamplingProvider, McpServerConfig,
    create_mcp_sampling_message,
};

const MAX_SAMPLING_STDIO_FRAME_BYTES: usize = 64 * 1024;
const MAX_SAMPLING_STDIO_ERROR_MESSAGE_CHARS: usize = 256;
const SAMPLING_REJECTION_PREFIX: &str = "sampling request rejected: ";

pub const MCP_SAMPLING_CREATE_MESSAGE_METHOD: &str = "sampling/createMessage";

pub fn parse_mcp_sampling_stdio_request_frame(
    frame: &str,
) -> Result<Option<(McpJsonRpcId, McpCreateMessageRequestParams)>, McpJsonRpcError> {
    if frame.len() > MAX_SAMPLING_STDIO_FRAME_BYTES {
        return Err(McpJsonRpcError::invalid_request("JSON-RPC frame too large"));
    }

    let request = serde_json::from_str::<McpJsonRpcRequest>(frame.trim_end())
        .map_err(|_| McpJsonRpcError::parse_error("invalid JSON-RPC frame"))?;
    if request.jsonrpc != "2.0" {
        return Err(McpJsonRpcError::invalid_request(
            "JSON-RPC version must be 2.0",
        ));
    }

    let Some(id) = request.id else {
        return Ok(None);
    };

    if request.method != MCP_SAMPLING_CREATE_MESSAGE_METHOD {
        return Err(McpJsonRpcError::method_not_found(
            request.method,
            "unsupported sampling stdio method",
        ));
    }

    let params = serde_json::from_value::<McpCreateMessageRequestParams>(request.params)
        .map_err(|_| McpJsonRpcError::invalid_params("invalid sampling/createMessage params"))?;

    Ok(Some((id, params)))
}

pub async fn handle_mcp_sampling_stdio_frame(
    provider: &dyn McpSamplingProvider,
    config: &McpServerConfig,
    frame: &str,
) -> String {
    let (id, request) = match parse_mcp_sampling_stdio_request_frame(frame) {
        Ok(Some(request)) => request,
        Ok(None) => return String::new(),
        Err(error) => return render_failure(McpJsonRpcId::Null, error),
    };

    match create_mcp_sampling_message(provider, config, request).await {
        Ok(result) => match serde_json::to_value(result) {
            Ok(value) => render_success(id, value),
            Err(_) => render_failure(id, McpJsonRpcError::internal_error("serialization failed")),
        },
        Err(error) => render_failure(id, mcp_error_to_json_rpc_error(error)),
    }
}

fn render_success(id: McpJsonRpcId, result: serde_json::Value) -> String {
    render_frame(&McpJsonRpcSuccess::new(id, result))
}

fn render_failure(id: McpJsonRpcId, error: McpJsonRpcError) -> String {
    render_frame(&McpJsonRpcFailure::new(id, error))
}

fn render_frame<T: serde::Serialize>(frame: &T) -> String {
    match serde_json::to_string(frame) {
        Ok(mut rendered) => {
            rendered.push('\n');
            rendered
        }
        Err(_) => {
            "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32603,\"message\":\"serialization failed\"}}\n"
                .to_string()
        }
    }
}

fn mcp_error_to_json_rpc_error(error: McpError) -> McpJsonRpcError {
    match error {
        McpError::InvalidServerRequest(message) if is_safe_sampling_rejection(&message) => {
            McpJsonRpcError {
                code: -32600,
                message: bound_public_error_message(&message),
            }
        }
        McpError::InvalidServerRequest(message) => McpJsonRpcError::invalid_request(message),
        McpError::Client(_) => McpJsonRpcError::internal_error("sampling provider failed"),
        McpError::InvalidConfig(message)
        | McpError::InvalidToolDescriptor(message)
        | McpError::ServerNotFound(message) => McpJsonRpcError::internal_error(message),
    }
}

fn is_safe_sampling_rejection(message: &str) -> bool {
    let Some(codes) = message.strip_prefix(SAMPLING_REJECTION_PREFIX) else {
        return false;
    };

    !codes.is_empty()
        && codes.split(", ").all(|code| {
            !code.is_empty() && code.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_')
        })
}

fn bound_public_error_message(message: &str) -> String {
    let mut chars = message.chars();
    let bounded = chars
        .by_ref()
        .take(MAX_SAMPLING_STDIO_ERROR_MESSAGE_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}
