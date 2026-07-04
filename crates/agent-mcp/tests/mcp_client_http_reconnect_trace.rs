use std::collections::BTreeMap;

use agent_mcp::{
    FixtureMcpClientHttpStreamTransport, FixtureMcpClientHttpTransport, McpClient as _,
    McpClientHttpReconnectAttempt, McpClientHttpReconnectAttemptOutcome,
    McpClientHttpReconnectFailureKind, McpClientHttpReconnectPolicy, McpClientHttpReconnectReport,
    McpClientHttpReplayDiagnostic, McpClientHttpResponse, McpClientHttpResponseEnvelope,
    McpClientHttpSessionMetadata, McpClientHttpStreamOpen, McpClientHttpStreamRequest, McpError,
    McpHttpClient, McpServerConfig, McpServerId, McpTransportConfig,
    parse_mcp_client_http_sse_events,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("http-reconnect-trace").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Reconnect Trace",
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
                    "name": "http-reconnect-trace-server",
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

fn replay_diagnostic() -> McpClientHttpReplayDiagnostic {
    let mut headers = BTreeMap::new();
    headers.insert(
        "Last-Event-ID".to_string(),
        "event-secret-token".to_string(),
    );
    let request =
        McpClientHttpStreamRequest::new("http://localhost/mcp", headers, 1_000).expect("request");
    let before = McpClientHttpSessionMetadata {
        session_id: Some("session-abc".to_string()),
        protocol_version: Some("2025-11-25".to_string()),
        last_event_id: Some("event-secret-token".to_string()),
    };
    let after = McpClientHttpSessionMetadata {
        session_id: Some("session-abc".to_string()),
        protocol_version: Some("2025-11-25".to_string()),
        last_event_id: Some("event-2".to_string()),
    };

    McpClientHttpReplayDiagnostic::from_reconnect_attempt(&before, &request, Some(&after))
}

#[test]
fn mcp_client_http_reconnect_trace_serializes_stable_shape() {
    let report = McpClientHttpReconnectReport::from_attempts(vec![
        McpClientHttpReconnectAttempt::retry(
            1,
            25,
            McpClientHttpReconnectFailureKind::RetryableTransport,
            replay_diagnostic(),
        ),
        McpClientHttpReconnectAttempt::success(2, replay_diagnostic()),
    ]);
    let value = serde_json::to_value(report).expect("json");

    assert_eq!(value["succeeded"], true);
    assert!(value["final_failure"].is_null());
    assert_eq!(value["attempts"][0]["attempt"], 1);
    assert_eq!(value["attempts"][0]["outcome"], "retry");
    assert_eq!(value["attempts"][0]["backoff_ms"], 25);
    assert_eq!(
        value["attempts"][0]["failure_summary"],
        "retryable MCP client HTTP transport failure"
    );
    assert_eq!(value["attempts"][1]["attempt"], 2);
    assert_eq!(value["attempts"][1]["outcome"], "success");
}

#[test]
fn mcp_client_http_reconnect_trace_failure_summary_is_redacted() {
    let report =
        McpClientHttpReconnectReport::from_attempts(vec![McpClientHttpReconnectAttempt::failed(
            1,
            McpClientHttpReconnectFailureKind::NonRetryableConfig,
            replay_diagnostic(),
        )]);
    let serialized = serde_json::to_string(&report).expect("json");

    assert!(!report.succeeded);
    assert_eq!(
        report.final_failure.as_deref(),
        Some("non-retryable MCP client HTTP config failure")
    );
    assert!(!serialized.contains("event-secret-token"));
    assert!(!serialized.contains("session-abc"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("api_key"));
}

#[test]
fn mcp_client_http_reconnect_trace_attempt_outcome_is_typed() {
    let success = McpClientHttpReconnectAttempt::success(1, replay_diagnostic());
    let retry = McpClientHttpReconnectAttempt::retry(
        2,
        50,
        McpClientHttpReconnectFailureKind::RetryableStatus,
        replay_diagnostic(),
    );
    let failed = McpClientHttpReconnectAttempt::failed(
        3,
        McpClientHttpReconnectFailureKind::NonRetryableValidation,
        replay_diagnostic(),
    );

    assert_eq!(
        success.outcome,
        McpClientHttpReconnectAttemptOutcome::Success
    );
    assert_eq!(retry.outcome, McpClientHttpReconnectAttemptOutcome::Retry);
    assert_eq!(retry.backoff_ms, Some(50));
    assert_eq!(failed.outcome, McpClientHttpReconnectAttemptOutcome::Failed);
}

#[tokio::test]
async fn mcp_client_http_reconnect_trace_runner_report_records_retry_then_success() {
    let config = config();
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        initialize_envelope(Some("event-secret-token")),
    )]));
    client.initialize(&config).await.expect("initialize");
    let mut stream_transport = FixtureMcpClientHttpStreamTransport::from_streams([
        Err(McpError::Client(
            "MCP client HTTP stream transport failed with sk-secret-token api_key".to_string(),
        )),
        Ok(stream_open(
            "id: event-2\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"ok\":true}}\n\n",
        )),
    ]);

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
    let serialized = serde_json::to_string(&report.reconnect_report).expect("json");

    assert_eq!(report.reconnect_report.attempts.len(), 2);
    assert!(report.reconnect_report.succeeded);
    assert_eq!(
        report.reconnect_report.attempts[0].outcome,
        McpClientHttpReconnectAttemptOutcome::Retry
    );
    assert_eq!(
        report.reconnect_report.attempts[1].outcome,
        McpClientHttpReconnectAttemptOutcome::Success
    );
    assert!(!serialized.contains("event-secret-token"));
    assert!(!serialized.contains("session-abc"));
    assert!(!serialized.contains("sk-secret-token"));
    assert!(!serialized.contains("api_key"));
}
