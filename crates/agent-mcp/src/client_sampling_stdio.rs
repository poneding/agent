use crate::{McpCreateMessageRequestParams, McpJsonRpcError, McpJsonRpcId, McpJsonRpcRequest};

const MAX_SAMPLING_STDIO_FRAME_BYTES: usize = 64 * 1024;

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
