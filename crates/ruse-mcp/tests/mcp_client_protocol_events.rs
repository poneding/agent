use ruse_mcp::{
    McpProtocolEventKind, McpProtocolEventStatus, McpServerConfig, McpServerId, McpTransportConfig,
    parse_mcp_protocol_event_frame,
};
use serde_json::json;

fn config() -> McpServerConfig {
    McpServerConfig::new(
        McpServerId::new("event-server").expect("server id"),
        "Event Server",
        McpTransportConfig::stdio("mcp-events"),
    )
    .expect("config")
}

#[test]
fn mcp_client_protocol_events_normalize_progress_summary() {
    let report = parse_mcp_protocol_event_frame(
        &config(),
        &json!({
            "jsonrpc": "2.0",
            "method": "notifications/progress",
            "params": {
                "progressToken": "token-value-that-must-not-leak",
                "progress": 3,
                "total": 10,
                "message": "Indexing workspace"
            }
        })
        .to_string(),
    );

    assert_eq!(report.event_kind, McpProtocolEventKind::Progress);
    assert_eq!(report.status, McpProtocolEventStatus::Accepted);
    assert_eq!(
        report.ui_summary.message.as_deref(),
        Some("Indexing workspace")
    );
    let progress = report
        .ui_summary
        .progress
        .as_ref()
        .expect("progress summary");
    assert_eq!(progress.progress, Some(3.0));
    assert_eq!(progress.total, Some(10.0));
    assert!(progress.progress_token_present);
    let json = serde_json::to_string(&report).expect("report json");
    assert!(!json.contains("token-value-that-must-not-leak"));
}

#[test]
fn mcp_client_protocol_events_redact_logging_payloads() {
    let report = parse_mcp_protocol_event_frame(
        &config(),
        &json!({
            "jsonrpc": "2.0",
            "method": "notifications/message",
            "params": {
                "level": "warning",
                "data": {
                    "message": "provider_payload bearer sk-secret-token"
                },
                "headers": {
                    "authorization": "Bearer sk-secret-token"
                }
            }
        })
        .to_string(),
    );

    assert_eq!(report.event_kind, McpProtocolEventKind::Logging);
    assert_eq!(report.ui_summary.level.as_deref(), Some("warning"));
    assert_eq!(report.ui_summary.message.as_deref(), Some("[redacted]"));
    assert!(report.ui_summary.redacted);
    let json = serde_json::to_string(&report).expect("report json");
    assert!(!json.contains("sk-secret-token"));
    assert!(!json.contains("Bearer"));
    assert!(!json.contains("authorization"));
}

#[test]
fn mcp_client_protocol_events_normalize_task_and_elicitation() {
    let task = parse_mcp_protocol_event_frame(
        &config(),
        &json!({
            "jsonrpc": "2.0",
            "method": "tasks/updated",
            "params": {
                "taskId": "task-123",
                "status": "running",
                "message": "Task advanced"
            }
        })
        .to_string(),
    );
    let elicitation = parse_mcp_protocol_event_frame(
        &config(),
        &json!({
            "jsonrpc": "2.0",
            "id": "elicitation-1",
            "method": "elicitation/create",
            "params": {
                "message": "Need user confirmation",
                "schema": {"type": "object"}
            }
        })
        .to_string(),
    );

    assert_eq!(task.event_kind, McpProtocolEventKind::Task);
    assert_eq!(task.ui_summary.task_ref.as_deref(), Some("task-123"));
    assert_eq!(task.ui_summary.status.as_deref(), Some("running"));
    assert!(!task.request_id_present);
    assert_eq!(elicitation.event_kind, McpProtocolEventKind::Elicitation);
    assert_eq!(
        elicitation.ui_summary.message.as_deref(),
        Some("Need user confirmation")
    );
    assert!(elicitation.request_id_present);
}

#[test]
fn mcp_client_protocol_events_accept_unknown_without_raw_payload() {
    let report = parse_mcp_protocol_event_frame(
        &config(),
        &json!({
            "jsonrpc": "2.0",
            "method": "vendor/rawThing",
            "params": {
                "raw_prompt": "secret prompt",
                "nested": {"api_key": "sk-secret"}
            }
        })
        .to_string(),
    );

    assert_eq!(report.event_kind, McpProtocolEventKind::Unknown);
    assert_eq!(report.status, McpProtocolEventStatus::Accepted);
    assert_eq!(report.ui_summary.field_count, 2);
    assert!(report.ui_summary.redacted);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "event_unknown_method")
    );
    let json = serde_json::to_string(&report).expect("report json");
    assert!(!json.contains("secret prompt"));
    assert!(!json.contains("sk-secret"));
    assert!(!json.contains("api_key"));
}

#[test]
fn mcp_client_protocol_events_reject_invalid_frame() {
    let report = parse_mcp_protocol_event_frame(&config(), "{not-json");

    assert_eq!(report.status, McpProtocolEventStatus::Rejected);
    assert_eq!(report.event_kind, McpProtocolEventKind::Unknown);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "event_parse_failed")
    );
}
