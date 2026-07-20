use std::collections::BTreeMap;

use ruse_mcp::{
    McpClientHttpOAuthTokenLease, McpClientHttpSessionMetadata, McpEnvValue, McpServerConfig,
    McpServerId, McpTransportConfig, build_mcp_client_http_request_with_oauth_token,
    build_mcp_client_http_stream_request_with_oauth_token,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("mcp-oauth-token-request").expect("server id")
}

fn streamable_config(headers: BTreeMap<String, McpEnvValue>) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "MCP OAuth Token Request",
        McpTransportConfig::StreamableHttp {
            url: "https://mcp.example.com".to_string(),
            headers,
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

fn lease(expires_at_unix_ms: Option<u64>) -> McpClientHttpOAuthTokenLease {
    McpClientHttpOAuthTokenLease::new(
        "request-secret-token",
        expires_at_unix_ms,
        vec!["mcp:tools".to_string()],
    )
    .expect("token lease")
}

#[test]
fn mcp_client_http_oauth_token_request_applies_to_post_request() {
    let request = build_mcp_client_http_request_with_oauth_token(
        &streamable_config(BTreeMap::new()),
        json!({"jsonrpc": "2.0"}),
        &lease(Some(2_000)),
        1_000,
    )
    .expect("request");

    assert_eq!(
        request.headers.get("Authorization").map(String::as_str),
        Some("Bearer request-secret-token")
    );
    assert_eq!(request.timeout_ms, 1_000);
}

#[test]
fn mcp_client_http_oauth_token_request_applies_to_get_stream_request() {
    let metadata = McpClientHttpSessionMetadata {
        session_id: Some("session-1".to_string()),
        protocol_version: Some("2025-11-25".to_string()),
        last_event_id: Some("event-1".to_string()),
    };
    let request = build_mcp_client_http_stream_request_with_oauth_token(
        &streamable_config(BTreeMap::new()),
        &metadata,
        &lease(Some(2_000)),
        1_000,
    )
    .expect("stream request");

    assert_eq!(
        request.headers.get("Authorization").map(String::as_str),
        Some("Bearer request-secret-token")
    );
    assert_eq!(
        request.headers.get("Last-Event-ID").map(String::as_str),
        Some("event-1")
    );
}

#[test]
fn mcp_client_http_oauth_token_request_allows_unknown_expiry() {
    let request = build_mcp_client_http_request_with_oauth_token(
        &streamable_config(BTreeMap::new()),
        json!({"jsonrpc": "2.0"}),
        &lease(None),
        1_000,
    )
    .expect("request");

    assert_eq!(
        request.headers.get("Authorization").map(String::as_str),
        Some("Bearer request-secret-token")
    );
}

#[test]
fn mcp_client_http_oauth_token_request_rejects_expired_token_without_leaking() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_string(),
        McpEnvValue::Literal("Bearer configured-secret-token".to_string()),
    );
    let error = build_mcp_client_http_request_with_oauth_token(
        &streamable_config(headers),
        json!({"jsonrpc": "2.0"}),
        &lease(Some(1_000)),
        1_000,
    )
    .expect_err("expired token");
    let message = error.to_string();

    assert!(message.contains("OAuth token lease is expired"));
    assert!(!message.contains("request-secret-token"));
    assert!(!message.contains("configured-secret-token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_request_rejects_header_conflicts_without_leaking() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_string(),
        McpEnvValue::Literal("Bearer configured-secret-token".to_string()),
    );
    let error = build_mcp_client_http_request_with_oauth_token(
        &streamable_config(headers),
        json!({"jsonrpc": "2.0"}),
        &lease(Some(2_000)),
        1_000,
    )
    .expect_err("conflict");
    let message = error.to_string();

    assert!(message.contains("authorization header conflicts"));
    assert!(!message.contains("request-secret-token"));
    assert!(!message.contains("configured-secret-token"));
    assert!(!message.contains("secret-token"));
}
