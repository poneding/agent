use ruse_mcp::{
    MCP_SERVER_PROTOCOL_VERSION, McpResourceExportRegistry, McpServerCapabilities,
    McpServerFixture, McpServerInfo, McpServerSession, McpServerSessionConfig,
    McpServerSessionStatus, McpServerState, McpToolExportRegistry,
};
use ruse_policy::StaticPolicyEngine;
use ruse_tool::ToolRegistry;
use serde_json::Value;

fn fixture() -> McpServerFixture {
    McpServerFixture::new(
        McpServerState::new(
            McpServerInfo::new("agent-session", "0.1.0", MCP_SERVER_PROTOCOL_VERSION)
                .expect("server info"),
            McpServerCapabilities {
                tools: true,
                prompts: false,
                resources: true,
            },
        ),
        ToolRegistry::default(),
        McpToolExportRegistry::default(),
        McpResourceExportRegistry::default(),
        StaticPolicyEngine::default(),
    )
}

fn response_value(frame: &str) -> Value {
    serde_json::from_str(frame.trim_end()).expect("json response")
}

#[test]
fn mcp_server_session_initialize_updates_status_and_summary() {
    let mut session = McpServerSession::new(fixture(), McpServerSessionConfig::default());

    let response =
        session.handle_frame(r#"{"jsonrpc":"2.0","id":"init","method":"initialize","params":{}}"#);
    let value = response_value(&response);
    let summary = session.summary();

    assert_eq!(session.status(), McpServerSessionStatus::Initialized);
    assert_eq!(value["result"]["initialized"], true);
    assert_eq!(summary.frames_received, 1);
    assert_eq!(summary.responses_sent, 1);
    assert_eq!(summary.error_frames, 0);
    assert_eq!(summary.close_reason, None);
}

#[test]
fn mcp_server_session_recovers_after_malformed_frame() {
    let mut session = McpServerSession::new(fixture(), McpServerSessionConfig::default());
    session.handle_frame(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);

    let malformed = response_value(&session.handle_frame("{not json"));
    let list = response_value(
        &session.handle_frame(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#),
    );
    let summary = session.summary();

    assert_eq!(malformed["error"]["code"], -32700);
    assert_eq!(list["result"]["tools"], Value::Array(Vec::new()));
    assert_eq!(session.status(), McpServerSessionStatus::Initialized);
    assert_eq!(summary.frames_received, 3);
    assert_eq!(summary.responses_sent, 3);
    assert_eq!(summary.error_frames, 1);
}

#[test]
fn mcp_server_session_enforces_frame_limit_after_serving_allowed_frame() {
    let mut session =
        McpServerSession::new(fixture(), McpServerSessionConfig::new(64 * 1024, Some(1)));

    let init = response_value(
        &session.handle_frame(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#),
    );
    let after_limit = response_value(
        &session.handle_frame(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#),
    );
    let summary = session.summary();

    assert_eq!(init["result"]["initialized"], true);
    assert_eq!(after_limit["error"]["code"], -32600);
    assert_eq!(session.status(), McpServerSessionStatus::Closed);
    assert_eq!(summary.frames_received, 1);
    assert_eq!(summary.responses_sent, 2);
    assert_eq!(summary.error_frames, 1);
    assert_eq!(
        summary.close_reason,
        Some("frame limit reached".to_string())
    );
}

#[test]
fn mcp_server_session_rejects_oversized_frame_without_echoing_input() {
    let mut session = McpServerSession::new(fixture(), McpServerSessionConfig::new(8, None));
    let oversized = "x".repeat(128);

    let response = session.handle_frame(&oversized);
    let value = response_value(&response);
    let summary = session.summary();

    assert_eq!(value["error"]["code"], -32600);
    assert_eq!(value["error"]["message"], "JSON-RPC frame too large");
    assert!(!response.contains(&oversized));
    assert_eq!(session.status(), McpServerSessionStatus::Created);
    assert_eq!(summary.frames_received, 1);
    assert_eq!(summary.responses_sent, 1);
    assert_eq!(summary.error_frames, 1);
}

#[test]
fn mcp_server_session_close_returns_deterministic_summary() {
    let mut session = McpServerSession::new(fixture(), McpServerSessionConfig::default());
    let summary = session.close("eof");

    assert_eq!(summary.status, McpServerSessionStatus::Closed);
    assert_eq!(summary.frames_received, 0);
    assert_eq!(summary.responses_sent, 0);
    assert_eq!(summary.error_frames, 0);
    assert_eq!(summary.close_reason, Some("eof".to_string()));
}
