use agent_mcp::{
    MCP_SAMPLING_CREATE_MESSAGE_METHOD, McpCreateMessageRequestParams, McpJsonRpcId,
    McpSamplingContent, McpSamplingMessage, parse_mcp_sampling_stdio_request_frame,
};
use serde_json::json;

#[test]
fn mcp_sampling_stdio_contracts_decode_create_message_request() {
    let frame = json!({
        "jsonrpc": "2.0",
        "id": "sample-1",
        "method": MCP_SAMPLING_CREATE_MESSAGE_METHOD,
        "params": {
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "summarize this"
                    }
                }
            ],
            "maxTokens": 128
        }
    })
    .to_string();

    let (id, params) = parse_mcp_sampling_stdio_request_frame(&frame)
        .expect("valid frame")
        .expect("request id");

    assert_eq!(id, McpJsonRpcId::String("sample-1".to_string()));
    assert_eq!(
        params,
        McpCreateMessageRequestParams::new(
            vec![McpSamplingMessage::user(McpSamplingContent::text(
                "summarize this"
            ))],
            128,
        )
    );
}

#[test]
fn mcp_sampling_stdio_contracts_notification_requires_no_response() {
    let frame = json!({
        "jsonrpc": "2.0",
        "method": MCP_SAMPLING_CREATE_MESSAGE_METHOD,
        "params": {
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "notify only"
                    }
                }
            ],
            "maxTokens": 64
        }
    })
    .to_string();

    let parsed = parse_mcp_sampling_stdio_request_frame(&frame).expect("valid notification");

    assert_eq!(parsed, None);
}

#[test]
fn mcp_sampling_stdio_contracts_reject_unsupported_method_safely() {
    let frame = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "sampling/sk-secret-method",
        "params": {}
    })
    .to_string();

    let error = parse_mcp_sampling_stdio_request_frame(&frame).expect_err("unsupported method");

    assert_eq!(error.code, -32601);
    assert_eq!(error.message, "[redacted]");
    assert!(!error.message.contains("sk-secret-method"));
}

#[test]
fn mcp_sampling_stdio_contracts_reject_invalid_params() {
    let frame = json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": MCP_SAMPLING_CREATE_MESSAGE_METHOD,
        "params": {
            "messages": "not an array",
            "maxTokens": 128
        }
    })
    .to_string();

    let error = parse_mcp_sampling_stdio_request_frame(&frame).expect_err("invalid params");

    assert_eq!(error.code, -32602);
    assert_eq!(error.message, "invalid sampling/createMessage params");
}
