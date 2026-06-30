use std::collections::BTreeMap;

use agent_mcp::{
    FixtureMcpClientHttpStreamTransport, FixtureMcpClientHttpTransport, McpClient as _,
    McpClientHttpReconnectPolicy, McpClientHttpResponse, McpClientHttpResponseEnvelope,
    McpClientHttpSessionMetadata, McpClientHttpStreamOpen, McpError, McpHttpClient,
    McpServerConfig, McpServerId, McpTransportConfig, parse_mcp_client_http_sse_events,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("http-stream-reconnect-runner").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Stream Reconnect Runner",
        McpTransportConfig::StreamableHttp {
            url: "http://localhost/mcp".to_string(),
            headers: BTreeMap::new(),
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

fn initialize_response(id: i64) -> McpClientHttpResponse {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-11-25",
                "serverInfo": {
                    "name": "http-stream-reconnect-runner-server",
                    "version": "1.0.0"
                },
                "capabilities": {
                    "tools": {}
                }
            }
        }),
    )
    .expect("response")
}

fn initialize_envelope(last_event_id: Option<&str>) -> McpClientHttpResponseEnvelope {
    McpClientHttpResponseEnvelope::new(
        initialize_response(1).into(),
        McpClientHttpSessionMetadata {
            session_id: Some("session-abc".to_string()),
            protocol_version: Some("2025-11-25".to_string()),
            last_event_id: last_event_id.map(ToString::to_string),
        },
    )
}

fn stream_open(body: &str) -> McpClientHttpStreamOpen {
    McpClientHttpStreamOpen::new(
        McpClientHttpSessionMetadata::default(),
        parse_mcp_client_http_sse_events(body).expect("events"),
    )
}

fn fast_policy(max_attempts: u32) -> McpClientHttpReconnectPolicy {
    McpClientHttpReconnectPolicy {
        max_attempts,
        initial_backoff_ms: 1,
        max_backoff_ms: 1,
    }
}

#[tokio::test]
async fn mcp_client_http_stream_reconnect_runner_succeeds_on_first_attempt() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(None),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport =
        FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream_open(
            "id: event-1\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}\n\n",
        ))]);

    let report = client
        .reconnect_stream(&config.id, &mut stream_transport, fast_policy(3))
        .await
        .expect("reconnect");
    let metadata = client.session_metadata_snapshot().await;

    assert_eq!(report.accepted_events, 1);
    assert_eq!(report.last_event_id.as_deref(), Some("event-1"));
    assert_eq!(metadata.last_event_id.as_deref(), Some("event-1"));
    assert_eq!(stream_transport.sent_requests().len(), 1);
}

#[tokio::test]
async fn mcp_client_http_stream_reconnect_runner_retries_retryable_failure_with_fresh_snapshot() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-1")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport = FixtureMcpClientHttpStreamTransport::from_streams([
        Err(McpError::Client(
            "MCP client HTTP stream transport failed with sk-secret-token".to_string(),
        )),
        Ok(stream_open(
            "id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
        )),
    ]);

    let report = client
        .reconnect_stream(&config.id, &mut stream_transport, fast_policy(3))
        .await
        .expect("reconnect");
    let metadata = client.session_metadata_snapshot().await;

    assert_eq!(report.accepted_events, 1);
    assert_eq!(report.last_event_id.as_deref(), Some("event-2"));
    assert_eq!(metadata.last_event_id.as_deref(), Some("event-2"));
    assert_eq!(stream_transport.sent_requests().len(), 2);
    for request in stream_transport.sent_requests() {
        assert_eq!(
            request.headers.get("Last-Event-ID").map(String::as_str),
            Some("event-1")
        );
    }
}

#[tokio::test]
async fn mcp_client_http_stream_reconnect_runner_stops_on_non_retryable_failure() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-1")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport = FixtureMcpClientHttpStreamTransport::from_streams([Err(
        McpError::InvalidConfig("secret header token".to_string()),
    )]);

    let error = client
        .reconnect_stream(&config.id, &mut stream_transport, fast_policy(3))
        .await
        .expect_err("non-retryable");
    let message = error.to_string();

    assert_eq!(stream_transport.sent_requests().len(), 1);
    assert!(message.contains("non-retryable MCP client HTTP config failure"));
    assert!(!message.contains("secret"));
    assert!(!message.contains("token"));
}

#[tokio::test]
async fn mcp_client_http_stream_reconnect_runner_stops_after_attempt_budget() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(None),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport = FixtureMcpClientHttpStreamTransport::from_streams([
        Err(McpError::Client(
            "MCP client HTTP stream transport failed with sk-secret-token-1".to_string(),
        )),
        Err(McpError::Client(
            "MCP client HTTP stream transport failed with sk-secret-token-2".to_string(),
        )),
    ]);

    let error = client
        .reconnect_stream(&config.id, &mut stream_transport, fast_policy(2))
        .await
        .expect_err("exhausted");
    let message = error.to_string();

    assert_eq!(stream_transport.sent_requests().len(), 2);
    assert!(message.contains("MCP client HTTP stream reconnect failed after 2 attempt(s)"));
    assert!(message.contains("retryable MCP client HTTP transport failure"));
    assert!(!message.contains("sk-secret-token"));
}

#[tokio::test]
async fn mcp_client_http_stream_reconnect_runner_requires_initialized_session() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::default());
    let mut stream_transport = FixtureMcpClientHttpStreamTransport::default();

    let error = client
        .reconnect_stream(&config.id, &mut stream_transport, fast_policy(1))
        .await
        .expect_err("not initialized");

    assert!(error.to_string().contains("not initialized"));
    assert!(stream_transport.sent_requests().is_empty());
}
