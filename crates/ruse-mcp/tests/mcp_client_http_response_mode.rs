use ruse_mcp::{
    McpClientHttpResponse, McpClientHttpResponseFrame, McpClientHttpResponseMode,
    McpClientHttpSseResponse, McpJsonRpcId, find_mcp_client_http_response_frame,
    parse_mcp_client_http_sse_events,
};
use serde_json::json;

#[test]
fn mcp_client_http_response_mode_preserves_json_response_behavior() {
    let response = McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "ok": true
            }
        }),
    )
    .expect("response");
    let mode = McpClientHttpResponseMode::Json(response);

    let frame = find_mcp_client_http_response_frame(&mode, &McpJsonRpcId::Number(3))
        .expect("matching frame");

    match frame {
        McpClientHttpResponseFrame::Success(success) => {
            assert_eq!(success.id, McpJsonRpcId::Number(3));
            assert_eq!(success.result["ok"], true);
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn mcp_client_http_response_mode_extracts_sse_success_frame() {
    let events = parse_mcp_client_http_sse_events(
        "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":4,\"result\":{\"tools\":[]}}\n\n",
    )
    .expect("events");
    let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events));

    let frame = find_mcp_client_http_response_frame(&mode, &McpJsonRpcId::Number(4))
        .expect("matching frame");

    match frame {
        McpClientHttpResponseFrame::Success(success) => {
            assert_eq!(success.id, McpJsonRpcId::Number(4));
            assert_eq!(success.result["tools"], json!([]));
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn mcp_client_http_response_mode_extracts_sse_failure_frame_with_redaction() {
    let events = parse_mcp_client_http_sse_events(
        "data: {\"jsonrpc\":\"2.0\",\"id\":\"call-1\",\"error\":{\"code\":-32000,\"message\":\"sk-secret-token\"}}\n\n",
    )
    .expect("events");
    let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events));

    let frame =
        find_mcp_client_http_response_frame(&mode, &McpJsonRpcId::String("call-1".to_string()))
            .expect("matching frame");

    match frame {
        McpClientHttpResponseFrame::Failure(failure) => {
            assert_eq!(failure.id, McpJsonRpcId::String("call-1".to_string()));
            assert_eq!(failure.error.code, -32000);
            assert_eq!(failure.error.message, "[redacted]");
        }
        other => panic!("expected failure, got {other:?}"),
    }
}

#[test]
fn mcp_client_http_response_mode_rejects_malformed_sse_json_without_leaking_data() {
    let events = parse_mcp_client_http_sse_events("data: {\"secret\":\"sk-secret-token\"\n\n")
        .expect("events");
    let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events));

    let error = find_mcp_client_http_response_frame(&mode, &McpJsonRpcId::Number(1))
        .expect_err("malformed data");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP response body"));
    assert!(!message.contains("sk-secret-token"));
}

#[test]
fn mcp_client_http_response_mode_rejects_missing_matching_id_safely() {
    let events = parse_mcp_client_http_sse_events(
        "data: {\"jsonrpc\":\"2.0\",\"id\":99,\"result\":{\"ok\":true}}\n\n",
    )
    .expect("events");
    let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events));

    let error = find_mcp_client_http_response_frame(&mode, &McpJsonRpcId::Number(1))
        .expect_err("missing id");
    let message = error.to_string();

    assert!(message.contains("matching response id missing"));
    assert!(!message.contains("99"));
    assert!(!message.contains("ok"));
}
