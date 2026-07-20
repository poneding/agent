use std::collections::BTreeMap;

use ruse_mcp::{
    FixtureMcpClientHttpTransport, McpClient as _, McpClientHttpResponse,
    McpClientHttpResponseEnvelope, McpClientHttpResponseMode, McpClientHttpSessionMetadata,
    McpClientHttpSseResponse, McpHttpClient, McpServerConfig, McpServerId, McpTransportConfig,
    parse_mcp_client_http_session_metadata, parse_mcp_client_http_sse_events,
};
use serde_json::{Value, json};

fn server_id() -> McpServerId {
    McpServerId::new("http-stream-resume").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Stream Resume",
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
                    "name": "http-stream-resume-server",
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

fn tools_sse_envelope(id: i64, event_id: &str, result: Value) -> McpClientHttpResponseEnvelope {
    let body = format!(
        "id: {event_id}\ndata: {}\n\n",
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        })
    );
    let mode = McpClientHttpResponseMode::Sse(McpClientHttpSseResponse::new(
        parse_mcp_client_http_sse_events(&body).expect("events"),
    ));
    let metadata =
        parse_mcp_client_http_session_metadata(&BTreeMap::new(), &mode).expect("metadata");
    McpClientHttpResponseEnvelope::new(mode, metadata)
}

fn initialize_envelope_with_metadata(id: i64) -> McpClientHttpResponseEnvelope {
    McpClientHttpResponseEnvelope::new(
        initialize_response(id).into(),
        McpClientHttpSessionMetadata {
            session_id: Some("session-abc".to_string()),
            protocol_version: Some("2025-11-25".to_string()),
            last_event_id: None,
        },
    )
}

#[tokio::test]
async fn mcp_client_http_stream_resume_snapshot_includes_session_headers() {
    let config = config();
    let transport =
        FixtureMcpClientHttpTransport::from_responses([Ok(initialize_envelope_with_metadata(1))]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    let snapshot = client
        .stream_request_snapshot(&config.id)
        .await
        .expect("stream snapshot");

    assert_eq!(snapshot.url, "http://localhost/mcp");
    assert_eq!(
        snapshot.headers.get("Mcp-Session-Id").map(String::as_str),
        Some("session-abc")
    );
    assert_eq!(
        snapshot
            .headers
            .get("MCP-Protocol-Version")
            .map(String::as_str),
        Some("2025-11-25")
    );
    assert!(!snapshot.headers.contains_key("Last-Event-ID"));
}

#[tokio::test]
async fn mcp_client_http_stream_resume_snapshot_includes_latest_checkpoint() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(initialize_envelope_with_metadata(1)),
        Ok(tools_sse_envelope(2, "event-2", json!({"tools": []}))),
    ]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    client.list_tools(&config.id).await.expect("tools");
    let snapshot = client
        .stream_request_snapshot(&config.id)
        .await
        .expect("stream snapshot");

    assert_eq!(
        snapshot.headers.get("Last-Event-ID").map(String::as_str),
        Some("event-2")
    );
}

#[tokio::test]
async fn mcp_client_http_stream_resume_snapshot_requires_initialized_session() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::default());

    let error = client
        .stream_request_snapshot(&config.id)
        .await
        .expect_err("not initialized");

    assert!(error.to_string().contains("not initialized"));
}

#[tokio::test]
async fn mcp_client_http_stream_resume_snapshot_rejects_wrong_server_id() {
    let config = config();
    let transport =
        FixtureMcpClientHttpTransport::from_responses([Ok(initialize_envelope_with_metadata(1))]);
    let client = McpHttpClient::new(transport);
    let other_server = McpServerId::new("other-server").expect("server id");

    client.initialize(&config).await.expect("initialize");
    let error = client
        .stream_request_snapshot(&other_server)
        .await
        .expect_err("wrong server");

    assert!(error.to_string().contains("not initialized"));
}

#[tokio::test]
async fn mcp_client_http_stream_resume_snapshot_does_not_mutate_metadata() {
    let config = config();
    let transport =
        FixtureMcpClientHttpTransport::from_responses([Ok(initialize_envelope_with_metadata(1))]);
    let client = McpHttpClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    let before = client.session_metadata_snapshot().await;
    let _snapshot = client
        .stream_request_snapshot(&config.id)
        .await
        .expect("stream snapshot");
    let after = client.session_metadata_snapshot().await;

    assert_eq!(before, after);
}
