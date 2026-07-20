use std::collections::BTreeMap;

use ruse_mcp::{
    McpClientHttpResponse, McpClientHttpResponseMode, McpClientHttpSseResponse,
    parse_mcp_client_http_session_metadata, parse_mcp_client_http_sse_events,
};
use serde_json::json;

#[test]
fn mcp_client_http_session_metadata_returns_empty_metadata() {
    let headers = BTreeMap::new();
    let response = json_response_mode();

    let metadata = parse_mcp_client_http_session_metadata(&headers, &response).expect("metadata");

    assert_eq!(metadata.session_id, None);
    assert_eq!(metadata.protocol_version, None);
    assert_eq!(metadata.last_event_id, None);
}

#[test]
fn mcp_client_http_session_metadata_parses_mixed_case_headers() {
    let mut headers = BTreeMap::new();
    headers.insert("McP-SeSsIoN-Id".to_string(), "session-123".to_string());
    headers.insert("MCP-Protocol-Version".to_string(), "2025-11-25".to_string());

    let metadata =
        parse_mcp_client_http_session_metadata(&headers, &json_response_mode()).expect("metadata");

    assert_eq!(metadata.session_id.as_deref(), Some("session-123"));
    assert_eq!(metadata.protocol_version.as_deref(), Some("2025-11-25"));
    assert_eq!(metadata.last_event_id, None);
}

#[test]
fn mcp_client_http_session_metadata_extracts_last_sse_event_id() {
    let events = parse_mcp_client_http_sse_events(
        "id: event-1\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n\nid: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{}}\n\n",
    )
    .expect("events");
    let response = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events));

    let metadata =
        parse_mcp_client_http_session_metadata(&BTreeMap::new(), &response).expect("metadata");

    assert_eq!(metadata.last_event_id.as_deref(), Some("event-2"));
}

#[test]
fn mcp_client_http_session_metadata_rejects_empty_header_value_without_leaking() {
    let mut headers = BTreeMap::new();
    headers.insert("Mcp-Session-Id".to_string(), "   ".to_string());

    let error = parse_mcp_client_http_session_metadata(&headers, &json_response_mode())
        .expect_err("invalid metadata");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP session metadata"));
    assert!(!message.contains("Mcp-Session-Id"));
}

#[test]
fn mcp_client_http_session_metadata_rejects_control_characters_without_leaking() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "mcp-session-id".to_string(),
        "session-sk-secret-token\n".to_string(),
    );

    let error = parse_mcp_client_http_session_metadata(&headers, &json_response_mode())
        .expect_err("invalid metadata");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP session metadata"));
    assert!(!message.contains("sk-secret-token"));
    assert!(!message.contains("session-sk"));
}

#[test]
fn mcp_client_http_session_metadata_rejects_oversized_event_id_without_leaking() {
    let body = format!(
        "id: {}\ndata: {{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{}}}}\n\n",
        "sk-secret-token".repeat(30)
    );
    let events = parse_mcp_client_http_sse_events(&body).expect("events");
    let response = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(events));

    let error = parse_mcp_client_http_session_metadata(&BTreeMap::new(), &response)
        .expect_err("oversized metadata");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP session metadata"));
    assert!(!message.contains("sk-secret-token"));
}

fn json_response_mode() -> McpClientHttpResponseMode {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {}
        }),
    )
    .expect("response")
    .into()
}
