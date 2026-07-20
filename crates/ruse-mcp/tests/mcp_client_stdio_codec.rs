use ruse_mcp::{
    McpClientStdioResponseFrame, McpJsonRpcError, McpJsonRpcId,
    parse_mcp_client_stdio_response_frame, render_mcp_client_stdio_request_frame,
};
use serde_json::{Value, json};

#[test]
fn mcp_client_stdio_codec_renders_initialize_request_frame() {
    let frame = render_mcp_client_stdio_request_frame(
        McpJsonRpcId::String("init-1".to_string()),
        "initialize",
        json!({
            "clientInfo": {
                "name": "agent",
                "version": "0.1.0"
            }
        }),
    )
    .expect("request frame");

    assert!(frame.ends_with('\n'));
    let value = serde_json::from_str::<Value>(frame.trim_end()).expect("json frame");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], "init-1");
    assert_eq!(value["method"], "initialize");
    assert_eq!(value["params"]["clientInfo"]["name"], "agent");
}

#[test]
fn mcp_client_stdio_codec_parses_success_response_frame() {
    let frame = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "result": {
            "tools": []
        }
    })
    .to_string();

    let response = parse_mcp_client_stdio_response_frame(&frame).expect("response frame");

    match response {
        McpClientStdioResponseFrame::Success(success) => {
            assert_eq!(success.id, McpJsonRpcId::Number(7));
            assert_eq!(success.result["tools"], json!([]));
        }
        other => panic!("expected success response, got {other:?}"),
    }
}

#[test]
fn mcp_client_stdio_codec_parses_failure_response_frame() {
    let frame = json!({
        "jsonrpc": "2.0",
        "id": "tools-1",
        "error": {
            "code": -32601,
            "message": "method not found"
        }
    })
    .to_string();

    let response = parse_mcp_client_stdio_response_frame(&frame).expect("response frame");

    match response {
        McpClientStdioResponseFrame::Failure(failure) => {
            assert_eq!(failure.id, McpJsonRpcId::String("tools-1".to_string()));
            assert_eq!(
                failure.error,
                McpJsonRpcError {
                    code: -32601,
                    message: "method not found".to_string()
                }
            );
        }
        other => panic!("expected failure response, got {other:?}"),
    }
}

#[test]
fn mcp_client_stdio_codec_rejects_notifications_as_responses() {
    let frame = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    })
    .to_string();

    let error = parse_mcp_client_stdio_response_frame(&frame).expect_err("notification rejected");

    assert!(error.to_string().contains("response id missing"));
}
