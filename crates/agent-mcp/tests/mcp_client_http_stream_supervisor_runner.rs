use std::collections::BTreeMap;

use agent_mcp::{
    FixtureMcpClientHttpStreamTransport, FixtureMcpClientHttpTransport, McpClient as _,
    McpClientHttpReconnectAttemptOutcome, McpClientHttpReconnectPolicy, McpClientHttpResponse,
    McpClientHttpResponseEnvelope, McpClientHttpSessionMetadata, McpClientHttpStreamOpen,
    McpClientHttpStreamSupervisorConfig, McpClientHttpStreamSupervisorStatus, McpError,
    McpHttpClient, McpServerConfig, McpServerId, McpTransportConfig,
    parse_mcp_client_http_sse_events,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("http-stream-supervisor-runner").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Stream Supervisor Runner",
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
                    "name": "http-stream-supervisor-runner-server",
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
            max_attempts: 3,
            initial_backoff_ms: 1,
            max_backoff_ms: 1,
        },
        max_ticks,
    }
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_runner_tick_succeeds_and_remains_running() {
    let config = config();
    let supervisor = supervisor_config(2);
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(None),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport =
        FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream_open(
            "id: event-1\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}\n\n",
        ))]);

    let tick = client
        .supervise_stream_tick(&supervisor, &mut stream_transport, 0)
        .await
        .expect("supervisor tick");
    let reconnect = tick.reconnect_report.expect("reconnect report");
    let metadata = client.session_metadata_snapshot().await;

    assert_eq!(tick.status, McpClientHttpStreamSupervisorStatus::Running);
    assert_eq!(tick.ticks_completed, 1);
    assert_eq!(reconnect.drain_report.accepted_events, 1);
    assert_eq!(
        reconnect.reconnect_report.attempts[0].outcome,
        McpClientHttpReconnectAttemptOutcome::Success
    );
    assert_eq!(metadata.last_event_id.as_deref(), Some("event-1"));
    assert_eq!(stream_transport.sent_requests().len(), 1);
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_runner_tick_retries_then_completes() {
    let config = config();
    let supervisor = supervisor_config(1);
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

    let tick = client
        .supervise_stream_tick(&supervisor, &mut stream_transport, 0)
        .await
        .expect("supervisor tick");
    let reconnect = tick.reconnect_report.expect("reconnect report");
    let serialized = serde_json::to_string(&reconnect.reconnect_report).expect("json");

    assert_eq!(tick.status, McpClientHttpStreamSupervisorStatus::Completed);
    assert_eq!(tick.ticks_completed, 1);
    assert_eq!(reconnect.reconnect_report.attempts.len(), 2);
    assert_eq!(
        reconnect.reconnect_report.attempts[0].outcome,
        McpClientHttpReconnectAttemptOutcome::Retry
    );
    assert_eq!(
        reconnect.reconnect_report.attempts[1].outcome,
        McpClientHttpReconnectAttemptOutcome::Success
    );
    assert!(!serialized.contains("sk-secret-token"));
    assert_eq!(stream_transport.sent_requests().len(), 2);
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_runner_stops_when_tick_budget_exhausted() {
    let config = config();
    let supervisor = supervisor_config(1);
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-1")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport =
        FixtureMcpClientHttpStreamTransport::from_streams([Ok(stream_open(
            "id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
        ))]);

    let tick = client
        .supervise_stream_tick(&supervisor, &mut stream_transport, 1)
        .await
        .expect("supervisor tick");
    let metadata = client.session_metadata_snapshot().await;

    assert_eq!(tick.status, McpClientHttpStreamSupervisorStatus::Completed);
    assert_eq!(tick.ticks_completed, 1);
    assert!(tick.reconnect_report.is_none());
    assert_eq!(metadata.last_event_id.as_deref(), Some("event-1"));
    assert!(stream_transport.sent_requests().is_empty());
}

#[tokio::test]
async fn mcp_client_http_stream_supervisor_runner_requires_initialized_session() {
    let supervisor = supervisor_config(1);
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::default());
    let mut stream_transport = FixtureMcpClientHttpStreamTransport::default();

    let error = client
        .supervise_stream_tick(&supervisor, &mut stream_transport, 0)
        .await
        .expect_err("not initialized");

    assert!(error.to_string().contains("not initialized"));
    assert!(stream_transport.sent_requests().is_empty());
}

#[test]
fn mcp_client_http_stream_supervisor_runner_tick_serializes_stable_shape() {
    let value = serde_json::to_value(agent_mcp::McpClientHttpStreamSupervisorTick {
        status: McpClientHttpStreamSupervisorStatus::Completed,
        ticks_completed: 3,
        reconnect_report: None,
    })
    .expect("json");

    assert_eq!(value["status"], "completed");
    assert_eq!(value["ticks_completed"], 3);
    assert!(value["reconnect_report"].is_null());
}
