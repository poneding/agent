use std::collections::BTreeMap;

use agent_mcp::{
    FixtureMcpClientHttpStreamTransport, FixtureMcpClientHttpTransport, McpClient as _,
    McpClientHttpReconnectPolicy, McpClientHttpResponse, McpClientHttpResponseEnvelope,
    McpClientHttpSessionMetadata, McpClientHttpStreamOpen, McpClientHttpStreamSupervisorConfig,
    McpClientHttpStreamSupervisorControl, McpClientHttpStreamSupervisorStatus, McpHttpClient,
    McpServerConfig, McpServerId, McpTransportConfig, parse_mcp_client_http_sse_events,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("http-stream-supervisor-cancellation").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Stream Supervisor Cancellation",
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
                    "name": "http-stream-supervisor-cancellation-server",
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

fn supervisor_config() -> McpClientHttpStreamSupervisorConfig {
    McpClientHttpStreamSupervisorConfig {
        server_id: server_id(),
        reconnect_policy: McpClientHttpReconnectPolicy {
            max_attempts: 1,
            initial_backoff_ms: 1,
            max_backoff_ms: 1,
        },
        max_ticks: 2,
    }
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_cancellation_stops_before_stream_side_effects() {
    let config = config();
    let supervisor = supervisor_config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-1")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport =
        FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream_open(
            "id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
        ))]);
    let control = McpClientHttpStreamSupervisorControl {
        cancelled: true,
        lease_valid: true,
    };

    let tick = client
        .supervise_stream_tick_with_control(&supervisor, &mut stream_transport, 0, &control)
        .await
        .expect("supervisor tick");
    let metadata = client.session_metadata_snapshot().await;

    assert_eq!(tick.status, McpClientHttpStreamSupervisorStatus::Cancelled);
    assert_eq!(tick.ticks_completed, 0);
    assert!(tick.reconnect_report.is_none());
    assert_eq!(
        tick.stop_reason.as_deref(),
        Some("MCP client HTTP stream supervision cancelled")
    );
    assert_eq!(metadata.last_event_id.as_deref(), Some("event-1"));
    assert!(stream_transport.sent_requests().is_empty());
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_cancellation_invalid_lease_stops_before_stream_side_effects()
 {
    let config = config();
    let supervisor = supervisor_config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-secret-token")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport =
        FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream_open(
            "id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
        ))]);
    let control = McpClientHttpStreamSupervisorControl {
        cancelled: false,
        lease_valid: false,
    };

    let tick = client
        .supervise_stream_tick_with_control(&supervisor, &mut stream_transport, 1, &control)
        .await
        .expect("supervisor tick");
    let metadata = client.session_metadata_snapshot().await;
    let serialized = serde_json::to_string(&tick).expect("json");

    assert_eq!(tick.status, McpClientHttpStreamSupervisorStatus::Failed);
    assert_eq!(tick.ticks_completed, 1);
    assert!(tick.reconnect_report.is_none());
    assert_eq!(
        tick.stop_reason.as_deref(),
        Some("MCP client HTTP stream supervisor lease is invalid")
    );
    assert_eq!(
        metadata.last_event_id.as_deref(),
        Some("event-secret-token")
    );
    assert!(stream_transport.sent_requests().is_empty());
    assert!(!serialized.contains("event-secret-token"));
    assert!(!serialized.contains("session-abc"));
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_cancellation_valid_control_proceeds_to_runner() {
    let config = config();
    let supervisor = supervisor_config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-1")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport =
        FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream_open(
            "id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
        ))]);
    let control = McpClientHttpStreamSupervisorControl::default();

    let tick = client
        .supervise_stream_tick_with_control(&supervisor, &mut stream_transport, 1, &control)
        .await
        .expect("supervisor tick");
    let metadata = client.session_metadata_snapshot().await;

    assert_eq!(tick.status, McpClientHttpStreamSupervisorStatus::Completed);
    assert_eq!(tick.ticks_completed, 2);
    assert!(tick.reconnect_report.is_some());
    assert!(tick.stop_reason.is_none());
    assert_eq!(metadata.last_event_id.as_deref(), Some("event-2"));
    assert_eq!(stream_transport.sent_requests().len(), 1);
}

#[test]
fn mcp_client_http_stream_supervisor_cancellation_control_serializes_stable_shape() {
    let control = McpClientHttpStreamSupervisorControl {
        cancelled: true,
        lease_valid: false,
    };
    let value = serde_json::to_value(control).expect("json");

    assert_eq!(value["cancelled"], true);
    assert_eq!(value["lease_valid"], false);
}
