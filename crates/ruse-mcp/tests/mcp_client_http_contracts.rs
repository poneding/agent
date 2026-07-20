use ruse_mcp::{
    McpClientHttpResponseFrame, McpJsonRpcId, parse_mcp_client_http_response_body,
    parse_mcp_client_http_response_text, render_mcp_client_http_request_body,
    validate_mcp_client_http_status,
};
use serde_json::json;

#[test]
fn mcp_client_http_contracts_render_request_body() {
    let body = render_mcp_client_http_request_body(
        McpJsonRpcId::String("init-1".to_string()),
        "initialize",
        json!({
            "clientInfo": {
                "name": "agent"
            }
        }),
    )
    .expect("request body");

    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], "init-1");
    assert_eq!(body["method"], "initialize");
    assert_eq!(body["params"]["clientInfo"]["name"], "agent");
}

#[test]
fn mcp_client_http_contracts_reject_null_request_id() {
    let error = render_mcp_client_http_request_body(McpJsonRpcId::Null, "tools/list", json!({}))
        .expect_err("null id");

    assert!(error.to_string().contains("request id cannot be null"));
}

#[test]
fn mcp_client_http_contracts_parse_success_response_body() {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "tools": []
        }
    });

    let response = parse_mcp_client_http_response_body(&body).expect("response body");

    match response {
        McpClientHttpResponseFrame::Success(success) => {
            assert_eq!(success.id, McpJsonRpcId::Number(2));
            assert_eq!(success.result["tools"], json!([]));
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn mcp_client_http_contracts_parse_failure_response_body() {
    let body = json!({
        "jsonrpc": "2.0",
        "id": "call-1",
        "error": {
            "code": -32601,
            "message": "method not found"
        }
    });

    let response = parse_mcp_client_http_response_body(&body).expect("response body");

    match response {
        McpClientHttpResponseFrame::Failure(failure) => {
            assert_eq!(failure.id, McpJsonRpcId::String("call-1".to_string()));
            assert_eq!(failure.error.code, -32601);
            assert_eq!(failure.error.message, "method not found");
        }
        other => panic!("expected failure, got {other:?}"),
    }
}

#[test]
fn mcp_client_http_contracts_redact_secret_like_failure_message() {
    let secret = "sk-secret-token from remote";
    let body = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "error": {
            "code": -32000,
            "message": secret
        }
    });

    let response = parse_mcp_client_http_response_body(&body).expect("response body");

    match response {
        McpClientHttpResponseFrame::Failure(failure) => {
            assert_eq!(failure.error.code, -32000);
            assert_eq!(failure.error.message, "[redacted]");
        }
        other => panic!("expected failure, got {other:?}"),
    }
}

#[test]
fn mcp_client_http_contracts_reject_malformed_response_text_without_leaking_body() {
    let secret_body = r#"{"jsonrpc":"2.0","id":1,"result":"sk-secret-token""#;

    let error = parse_mcp_client_http_response_text(secret_body).expect_err("malformed response");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP response body"));
    assert!(!message.contains("sk-secret-token"));
}

#[test]
fn mcp_client_http_contracts_reject_non_success_status_without_leaking_body() {
    let error = validate_mcp_client_http_status(503).expect_err("status");
    let message = error.to_string();

    assert!(message.contains("HTTP status 503"));
    assert!(!message.contains("token"));
    assert!(!message.contains("secret"));
}
