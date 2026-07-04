use std::collections::BTreeMap;

use agent_mcp::{
    FixtureMcpClientHttpStreamTransport, FixtureMcpClientHttpTransport, McpClient as _,
    McpClientHttpReconnectPolicy, McpClientHttpResponse, McpClientHttpResponseEnvelope,
    McpClientHttpSessionMetadata, McpClientHttpStreamOpen, McpClientHttpStreamSupervisorConfig,
    McpClientHttpStreamSupervisorControl, McpClientHttpStreamSupervisorReport,
    McpClientHttpStreamSupervisorStatus, McpHttpClient, McpServerConfig, McpServerId,
    McpTransportConfig, parse_mcp_client_http_sse_events,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("http-stream-supervisor-reports").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Stream Supervisor Reports",
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
                    "name": "http-stream-supervisor-reports-server",
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

fn supervisor_config(max_ticks: u32) -> McpClientHttpStreamSupervisorConfig {
    McpClientHttpStreamSupervisorConfig {
        server_id: server_id(),
        reconnect_policy: McpClientHttpReconnectPolicy {
            max_attempts: 2,
            initial_backoff_ms: 1,
            max_backoff_ms: 1,
        },
        max_ticks,
    }
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_reports_success_report_is_redacted() {
    let config = config();
    let supervisor = supervisor_config(1);
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-secret-token")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport =
        FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream_open(
            "id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
        ))]);

    let tick = client
        .supervise_stream_tick(&supervisor, &mut stream_transport, 0)
        .await
        .expect("supervisor tick");
    let report = McpClientHttpStreamSupervisorReport::from_tick(&tick);
    let serialized = serde_json::to_string(&report).expect("json");

    assert_eq!(
        report.status,
        McpClientHttpStreamSupervisorStatus::Completed
    );
    assert_eq!(report.ticks_completed, 1);
    assert!(report.terminal_summary.is_none());
    assert!(report.reconnect.is_some());
    assert!(!serialized.contains("event-secret-token"));
    assert!(!serialized.contains("event-2"));
    assert!(!serialized.contains("session-abc"));
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_reports_cancelled_report_has_public_summary() {
    let config = config();
    let supervisor = supervisor_config(2);
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-secret-token")),
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
    let report = McpClientHttpStreamSupervisorReport::from_tick(&tick);
    let serialized = serde_json::to_string(&report).expect("json");

    assert_eq!(
        report.status,
        McpClientHttpStreamSupervisorStatus::Cancelled
    );
    assert_eq!(
        report.terminal_summary.as_deref(),
        Some("MCP client HTTP stream supervision cancelled")
    );
    assert!(report.reconnect.is_none());
    assert!(!serialized.contains("event-secret-token"));
    assert!(!serialized.contains("session-abc"));
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_reports_failed_lease_report_is_redacted() {
    let config = config();
    let supervisor = supervisor_config(2);
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
    let report = McpClientHttpStreamSupervisorReport::from_tick(&tick);
    let serialized = serde_json::to_string(&report).expect("json");

    assert_eq!(report.status, McpClientHttpStreamSupervisorStatus::Failed);
    assert_eq!(
        report.terminal_summary.as_deref(),
        Some("MCP client HTTP stream supervisor lease is invalid")
    );
    assert!(report.reconnect.is_none());
    assert!(!serialized.contains("event-secret-token"));
    assert!(!serialized.contains("session-abc"));
}

#[test]
fn mcp_client_http_stream_supervisor_reports_serialize_stable_shape() {
    let tick = agent_mcp::McpClientHttpStreamSupervisorTick::stopped(
        McpClientHttpStreamSupervisorStatus::Cancelled,
        4,
        "MCP client HTTP stream supervision cancelled",
    );
    let report = McpClientHttpStreamSupervisorReport::from_tick(&tick);
    let value = serde_json::to_value(report).expect("json");

    assert_eq!(value["status"], "cancelled");
    assert_eq!(value["ticks_completed"], 4);
    assert_eq!(
        value["terminal_summary"],
        "MCP client HTTP stream supervision cancelled"
    );
    assert!(value["reconnect"].is_null());
}
