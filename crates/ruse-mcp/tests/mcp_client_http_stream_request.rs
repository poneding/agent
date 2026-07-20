use std::collections::BTreeMap;

use ruse_mcp::{
    McpClientHttpSessionMetadata, McpClientHttpStreamRequest, McpEnvValue, McpServerConfig,
    McpServerId, McpTransportConfig,
};

fn server_id() -> McpServerId {
    McpServerId::new("http-stream-request").expect("server id")
}

fn streamable_config(url: String, headers: BTreeMap<String, McpEnvValue>) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Stream Request",
        McpTransportConfig::StreamableHttp { url, headers },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

#[test]
fn mcp_client_http_stream_request_from_config_resolves_literal_headers() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "x-test".to_string(),
        McpEnvValue::Literal("fixture".to_string()),
    );
    let config = streamable_config("http://localhost/mcp".to_string(), headers);

    let request = McpClientHttpStreamRequest::from_config_with_session_metadata(
        &config,
        &McpClientHttpSessionMetadata::default(),
    )
    .expect("stream request");

    assert_eq!(request.url, "http://localhost/mcp");
    assert_eq!(request.headers["x-test"], "fixture");
    assert_eq!(request.timeout_ms, 1_000);
}

#[test]
fn mcp_client_http_stream_request_includes_resume_metadata_headers() {
    let config = streamable_config("http://localhost/mcp".to_string(), BTreeMap::new());
    let metadata = McpClientHttpSessionMetadata {
        session_id: Some("session-abc".to_string()),
        protocol_version: Some("2025-11-25".to_string()),
        last_event_id: Some("event-9".to_string()),
    };

    let request = McpClientHttpStreamRequest::from_config_with_session_metadata(&config, &metadata)
        .expect("stream request");

    assert_eq!(
        request.headers.get("Mcp-Session-Id").map(String::as_str),
        Some("session-abc")
    );
    assert_eq!(
        request
            .headers
            .get("MCP-Protocol-Version")
            .map(String::as_str),
        Some("2025-11-25")
    );
    assert_eq!(
        request.headers.get("Last-Event-ID").map(String::as_str),
        Some("event-9")
    );
}

#[test]
fn mcp_client_http_stream_request_omits_absent_resume_metadata_headers() {
    let config = streamable_config("http://localhost/mcp".to_string(), BTreeMap::new());

    let request = McpClientHttpStreamRequest::from_config_with_session_metadata(
        &config,
        &McpClientHttpSessionMetadata::default(),
    )
    .expect("stream request");

    assert!(!request.headers.contains_key("Mcp-Session-Id"));
    assert!(!request.headers.contains_key("MCP-Protocol-Version"));
    assert!(!request.headers.contains_key("Last-Event-ID"));
}

#[test]
fn mcp_client_http_stream_request_rejects_invalid_configs_before_request() {
    let disabled =
        streamable_config("http://localhost/mcp".to_string(), BTreeMap::new()).disabled();
    let disabled_error = McpClientHttpStreamRequest::from_config_with_session_metadata(
        &disabled,
        &McpClientHttpSessionMetadata::default(),
    )
    .expect_err("disabled");
    assert!(disabled_error.to_string().contains("disabled"));

    let non_http = McpServerConfig::new(
        server_id(),
        "Stdio",
        McpTransportConfig::stdio("mcp-server"),
    )
    .expect("config");
    let non_http_error = McpClientHttpStreamRequest::from_config_with_session_metadata(
        &non_http,
        &McpClientHttpSessionMetadata::default(),
    )
    .expect_err("non-http");
    assert!(non_http_error.to_string().contains("streamable http"));

    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_string(),
        McpEnvValue::SecretRef("mcp.token".to_string()),
    );
    let secret_ref = streamable_config("http://localhost/mcp".to_string(), headers);
    let secret_error = McpClientHttpStreamRequest::from_config_with_session_metadata(
        &secret_ref,
        &McpClientHttpSessionMetadata::default(),
    )
    .expect_err("secret ref");
    let message = secret_error.to_string();
    assert!(message.contains("unresolved secret ref"));
    assert!(!message.contains("authorization"));
    assert!(!message.contains("mcp.token"));
}

#[test]
fn mcp_client_http_stream_request_rejects_resume_header_conflicts_safely() {
    for reserved_header in ["mcp-session-id", "MCP-Protocol-Version", "last-event-id"] {
        let mut headers = BTreeMap::new();
        headers.insert(
            reserved_header.to_string(),
            McpEnvValue::Literal("configured-secret-token".to_string()),
        );
        let config = streamable_config("http://localhost/mcp".to_string(), headers);
        let metadata = McpClientHttpSessionMetadata {
            session_id: Some("session-secret-token".to_string()),
            protocol_version: Some("2025-11-25".to_string()),
            last_event_id: Some("event-secret-token".to_string()),
        };

        let error =
            McpClientHttpStreamRequest::from_config_with_session_metadata(&config, &metadata)
                .expect_err("header conflict");
        let message = error.to_string();

        assert!(message.contains("session header conflicts"));
        assert!(!message.contains("configured-secret-token"));
        assert!(!message.contains("session-secret-token"));
        assert!(!message.contains("event-secret-token"));
    }
}
