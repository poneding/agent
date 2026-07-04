use std::collections::BTreeMap;

use agent_mcp::{
    McpClientHttpAuthorization, McpClientHttpRequest, McpClientHttpSessionMetadata,
    McpClientHttpStreamRequest, McpEnvValue, McpServerConfig, McpServerId, McpTransportConfig,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("mcp-bearer").expect("server id")
}

fn streamable_config(headers: BTreeMap<String, McpEnvValue>) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "MCP Bearer",
        McpTransportConfig::StreamableHttp {
            url: "https://mcp.example.com".to_string(),
            headers,
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

#[test]
fn mcp_client_http_bearer_authorization_applies_to_post_request() {
    let authorization = McpClientHttpAuthorization::new("post-secret-token").expect("auth");
    let request = McpClientHttpRequest::from_config_with_authorization(
        &streamable_config(BTreeMap::new()),
        json!({"jsonrpc": "2.0"}),
        &authorization,
    )
    .expect("request");

    assert_eq!(
        request.headers.get("Authorization").map(String::as_str),
        Some("Bearer post-secret-token")
    );
    assert_eq!(request.timeout_ms, 1_000);
}

#[test]
fn mcp_client_http_bearer_authorization_applies_to_get_stream_request() {
    let authorization = McpClientHttpAuthorization::new("stream-secret-token").expect("auth");
    let metadata = McpClientHttpSessionMetadata {
        session_id: Some("session-1".to_string()),
        protocol_version: Some("2025-11-25".to_string()),
        last_event_id: Some("event-1".to_string()),
    };
    let request = McpClientHttpStreamRequest::from_config_with_session_metadata_and_authorization(
        &streamable_config(BTreeMap::new()),
        &metadata,
        &authorization,
    )
    .expect("stream request");

    assert_eq!(
        request.headers.get("Authorization").map(String::as_str),
        Some("Bearer stream-secret-token")
    );
    assert_eq!(
        request.headers.get("Last-Event-ID").map(String::as_str),
        Some("event-1")
    );
}

#[test]
fn mcp_client_http_bearer_authorization_rejects_empty_tokens_without_leaking() {
    let error = McpClientHttpAuthorization::new("  ").expect_err("empty token");
    let message = error.to_string();

    assert!(message.contains("bearer authorization is empty"));
    assert!(!message.contains("  "));
}

#[test]
fn mcp_client_http_bearer_authorization_rejects_header_conflicts_without_leaking() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_string(),
        McpEnvValue::Literal("Bearer configured-secret-token".to_string()),
    );
    let authorization = McpClientHttpAuthorization::new("caller-secret-token").expect("auth");
    let error = McpClientHttpRequest::from_config_with_authorization(
        &streamable_config(headers),
        json!({"jsonrpc": "2.0"}),
        &authorization,
    )
    .expect_err("conflict");
    let message = error.to_string();

    assert!(message.contains("authorization header conflicts"));
    assert!(!message.contains("configured-secret-token"));
    assert!(!message.contains("caller-secret-token"));
}

#[test]
fn mcp_client_http_bearer_authorization_debug_is_redacted() {
    let authorization = McpClientHttpAuthorization::new("debug-secret-token").expect("auth");
    let debug = format!("{authorization:?}");

    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("debug-secret-token"));
    assert!(!debug.contains("secret-token"));
}
