use std::collections::BTreeMap;

use ruse_mcp::{
    FixtureMcpClientHttpTransport, McpClient as _, McpClientHttpResponse,
    McpClientHttpResponseEnvelope, McpClientHttpSessionMetadata, McpClientHttpStreamOpen,
    McpHttpClient, McpServerConfig, McpServerId, McpTransportConfig,
    parse_mcp_client_http_sse_events,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("http-stream-drain").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Stream Drain",
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
                    "name": "http-stream-drain-server",
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

fn initialize_envelope_with_metadata(
    id: i64,
    last_event_id: Option<&str>,
) -> McpClientHttpResponseEnvelope {
    McpClientHttpResponseEnvelope::new(
        initialize_response(id).into(),
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

#[tokio::test]
async fn mcp_client_http_stream_drain_empty_stream_without_mutation() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope_with_metadata(1, None),
    )]));
    client.initialize(&config).await.expect("initialize");

    let report = client
        .record_stream_open(
            &config.id,
            McpClientHttpStreamOpen::new(McpClientHttpSessionMetadata::default(), Vec::new()),
        )
        .await
        .expect("drain");
    let metadata = client.session_metadata_snapshot().await;

    assert_eq!(report.accepted_events, 0);
    assert_eq!(report.last_event_id, None);
    assert_eq!(metadata.last_event_id, None);
}

#[tokio::test]
async fn mcp_client_http_stream_drain_checkpoints_last_accepted_event_id() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope_with_metadata(1, None),
    )]));
    client.initialize(&config).await.expect("initialize");
    let stream = stream_open(
        "id: event-1\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}\n\n\
         id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
    );

    let report = client
        .record_stream_open(&config.id, stream)
        .await
        .expect("drain");
    let metadata = client.session_metadata_snapshot().await;

    assert_eq!(report.accepted_events, 2);
    assert_eq!(report.last_event_id.as_deref(), Some("event-2"));
    assert_eq!(metadata.last_event_id.as_deref(), Some("event-2"));
}

#[tokio::test]
async fn mcp_client_http_stream_drain_rejects_malformed_event_without_mutation() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope_with_metadata(1, Some("prior-event")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let stream = stream_open("id: event-secret-token\ndata: sk-secret-token api_key\n\n");

    let error = client
        .record_stream_open(&config.id, stream)
        .await
        .expect_err("malformed event");
    let metadata = client.session_metadata_snapshot().await;
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP response body"));
    assert!(!message.contains("sk-secret-token"));
    assert!(!message.contains("api_key"));
    assert!(!message.contains("event-secret-token"));
    assert_eq!(metadata.last_event_id.as_deref(), Some("prior-event"));
}

#[tokio::test]
async fn mcp_client_http_stream_drain_requires_initialized_session() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::default());
    let stream = McpClientHttpStreamOpen::new(McpClientHttpSessionMetadata::default(), Vec::new());

    let error = client
        .record_stream_open(&config.id, stream)
        .await
        .expect_err("not initialized");

    assert!(error.to_string().contains("not initialized"));
}

#[tokio::test]
async fn mcp_client_http_stream_drain_rejects_wrong_server_id() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope_with_metadata(1, None),
    )]));
    let other_server = McpServerId::new("other-server").expect("server id");
    let stream = McpClientHttpStreamOpen::new(McpClientHttpSessionMetadata::default(), Vec::new());

    client.initialize(&config).await.expect("initialize");
    let error = client
        .record_stream_open(&other_server, stream)
        .await
        .expect_err("wrong server");

    assert!(error.to_string().contains("not initialized"));
}
