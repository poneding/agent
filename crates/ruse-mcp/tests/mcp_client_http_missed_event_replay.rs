use std::collections::BTreeMap;

use ruse_mcp::{
    FixtureMcpClientHttpStreamTransport, FixtureMcpClientHttpTransport, McpClient as _,
    McpClientHttpReconnectPolicy, McpClientHttpReplayDiagnostic, McpClientHttpReplayStatus,
    McpClientHttpResponse, McpClientHttpResponseEnvelope, McpClientHttpSessionMetadata,
    McpClientHttpStreamOpen, McpClientHttpStreamRequest, McpHttpClient, McpServerConfig,
    McpServerId, McpTransportConfig, parse_mcp_client_http_sse_events,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("http-missed-event-replay").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Missed Event Replay",
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
                    "name": "http-missed-event-replay-server",
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

fn request_with_last_event_id(last_event_id: Option<&str>) -> McpClientHttpStreamRequest {
    let mut headers = BTreeMap::new();
    if let Some(last_event_id) = last_event_id {
        headers.insert("Last-Event-ID".to_string(), last_event_id.to_string());
    }
    McpClientHttpStreamRequest::new("http://localhost/mcp", headers, 1_000).expect("request")
}

fn metadata(last_event_id: Option<&str>) -> McpClientHttpSessionMetadata {
    McpClientHttpSessionMetadata {
        session_id: Some("session-abc".to_string()),
        protocol_version: Some("2025-11-25".to_string()),
        last_event_id: last_event_id.map(ToString::to_string),
    }
}

#[test]
fn mcp_client_http_missed_event_replay_no_checkpoint_means_no_resume_request() {
    let diagnostic = McpClientHttpReplayDiagnostic::from_reconnect_attempt(
        &metadata(None),
        &request_with_last_event_id(None),
        Some(&metadata(None)),
    );

    assert!(!diagnostic.requested_resume);
    assert!(!diagnostic.checkpoint_advanced);
    assert!(!diagnostic.checkpoint_before.event_id_present);
    assert!(!diagnostic.requested_checkpoint.event_id_present);
    assert_eq!(
        diagnostic.replay_status,
        McpClientHttpReplayStatus::NotRequested
    );
}

#[test]
fn mcp_client_http_missed_event_replay_existing_checkpoint_reports_resume_without_raw_id() {
    let diagnostic = McpClientHttpReplayDiagnostic::from_reconnect_attempt(
        &metadata(Some("event-secret-token")),
        &request_with_last_event_id(Some("event-secret-token")),
        Some(&metadata(Some("event-secret-token"))),
    );
    let serialized = serde_json::to_string(&diagnostic).expect("json");

    assert!(diagnostic.requested_resume);
    assert!(!diagnostic.checkpoint_advanced);
    assert!(diagnostic.checkpoint_before.event_id_present);
    assert!(diagnostic.requested_checkpoint.event_id_present);
    assert!(diagnostic.checkpoint_after.event_id_present);
    assert_eq!(
        diagnostic.replay_status,
        McpClientHttpReplayStatus::Requested
    );
    assert!(!serialized.contains("event-secret-token"));
    assert!(!serialized.contains("secret"));
    assert!(!serialized.contains("token"));
}

#[test]
fn mcp_client_http_missed_event_replay_reports_checkpoint_advancement() {
    let diagnostic = McpClientHttpReplayDiagnostic::from_reconnect_attempt(
        &metadata(Some("event-1")),
        &request_with_last_event_id(Some("event-1")),
        Some(&metadata(Some("event-2"))),
    );

    assert!(diagnostic.requested_resume);
    assert!(diagnostic.checkpoint_advanced);
    assert_eq!(
        diagnostic.replay_status,
        McpClientHttpReplayStatus::CheckpointAdvanced
    );
}

#[test]
fn mcp_client_http_missed_event_replay_failure_keeps_unknown_status() {
    let diagnostic = McpClientHttpReplayDiagnostic::from_reconnect_attempt(
        &metadata(Some("event-secret-token")),
        &request_with_last_event_id(Some("event-secret-token")),
        None,
    );
    let serialized = serde_json::to_string(&diagnostic).expect("json");

    assert!(diagnostic.requested_resume);
    assert!(!diagnostic.checkpoint_advanced);
    assert_eq!(diagnostic.replay_status, McpClientHttpReplayStatus::Unknown);
    assert!(!diagnostic.checkpoint_after.event_id_present);
    assert!(!serialized.contains("event-secret-token"));
}

#[tokio::test]
async fn mcp_client_http_missed_event_replay_reconnect_report_includes_safe_diagnostic() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-secret-token")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport =
        FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream_open(
            "id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
        ))]);

    let report = client
        .reconnect_stream_with_replay_diagnostic(
            &config.id,
            &mut stream_transport,
            McpClientHttpReconnectPolicy {
                max_attempts: 2,
                initial_backoff_ms: 1,
                max_backoff_ms: 1,
            },
        )
        .await
        .expect("reconnect");
    let serialized = serde_json::to_string(&report).expect("json");

    assert_eq!(report.drain_report.accepted_events, 1);
    assert!(report.replay_diagnostic.requested_resume);
    assert!(report.replay_diagnostic.checkpoint_advanced);
    assert!(!serialized.contains("event-secret-token"));
}
