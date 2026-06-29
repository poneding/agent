use agent_mcp::{
    MCP_SERVER_PROTOCOL_VERSION, McpResourceExportRegistry, McpServerCapabilities,
    McpServerFixture, McpServerInfo, McpServerSession, McpServerSessionConfig, McpServerState,
    McpServerTraceEntry, McpServerTraceSnapshot, McpToolExportRegistry,
    format_mcp_server_trace_summary,
};
use agent_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use agent_tool::ToolRegistry;

fn fixture(policy: StaticPolicyEngine, resources: McpResourceExportRegistry) -> McpServerFixture {
    McpServerFixture::new(
        McpServerState::new(
            McpServerInfo::new("agent-trace", "0.1.0", MCP_SERVER_PROTOCOL_VERSION)
                .expect("server info"),
            McpServerCapabilities {
                tools: true,
                prompts: false,
                resources: true,
            },
        ),
        ToolRegistry::default(),
        McpToolExportRegistry::default(),
        resources,
        policy,
    )
}

fn empty_fixture() -> McpServerFixture {
    fixture(
        StaticPolicyEngine::default(),
        McpResourceExportRegistry::default(),
    )
}

fn deny_read_policy(uri: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, uri).expect("scope"),
        PolicyDecision::Deny {
            reason: "fixture deny with api_key sk-secret".to_string(),
        },
    )
}

#[test]
fn mcp_server_trace_entry_serializes_safe_contract() {
    let entry = McpServerTraceEntry::new("tools/list", "ok", true);
    let value = serde_json::to_value(&entry).expect("trace json");

    assert_eq!(value["method"], "tools/list");
    assert_eq!(value["outcome"], "ok");
    assert_eq!(value["initialized"], true);
}

#[test]
fn mcp_server_trace_redacts_secret_like_methods_and_bounds_summary() {
    let entries = vec![
        McpServerTraceEntry::new("initialize", "ok", true),
        McpServerTraceEntry::new("tools/call/api_key/sk-secret", "denied", true),
        McpServerTraceEntry::new("resources/list", "ok", true),
    ];

    let snapshot = McpServerTraceSnapshot::from_entries(&entries, 2);
    let rendered = format_mcp_server_trace_summary(&snapshot);

    assert_eq!(snapshot.total_entries, 3);
    assert_eq!(snapshot.error_entries, 1);
    assert!(snapshot.truncated);
    assert_eq!(snapshot.entries.len(), 2);
    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-secret"));
    assert!(!rendered.contains("api_key"));
}

#[test]
fn mcp_server_trace_summary_records_success_and_malformed_without_raw_frame() {
    let mut session = McpServerSession::new(empty_fixture(), McpServerSessionConfig::default());

    session.handle_frame(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    session.handle_frame(
        r#"{"jsonrpc":"2.0","id":2,"method":"secret/token","params":{"api_key":"sk-secret"}}"#,
    );
    session.handle_frame("{not json api_key sk-secret");
    let summary = session.summary();
    let rendered = format_mcp_server_trace_summary(&summary.trace);

    assert_eq!(summary.trace.total_entries, 3);
    assert_eq!(summary.trace.error_entries, 2);
    assert_eq!(summary.trace.entries[0].method, "initialize");
    assert_eq!(summary.trace.entries[0].outcome, "ok");
    assert_eq!(summary.trace.entries[1].method, "[redacted]");
    assert_eq!(summary.trace.entries[1].outcome, "error:method_not_found");
    assert_eq!(summary.trace.entries[2].method, "malformed");
    assert_eq!(summary.trace.entries[2].outcome, "error:parse");
    assert!(!rendered.contains("sk-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("{not json"));
}

#[test]
fn mcp_server_trace_records_policy_denied_resource_read_without_contents() {
    let mut resources = McpResourceExportRegistry::default();
    resources
        .export_resource(
            "agent://readme",
            "README",
            Some("text/plain"),
            "private resource content",
        )
        .expect("export resource");
    let mut session = McpServerSession::new(
        fixture(deny_read_policy("agent://readme"), resources),
        McpServerSessionConfig::default(),
    );

    session.handle_frame(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    let response = session.handle_frame(
        r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"agent://readme"}}"#,
    );
    let summary = session.summary();
    let rendered = format_mcp_server_trace_summary(&summary.trace);

    assert!(response.contains("policy denied MCP resource export"));
    assert_eq!(summary.trace.total_entries, 2);
    assert_eq!(summary.trace.error_entries, 1);
    assert_eq!(summary.trace.entries[1].method, "resources/read");
    assert_eq!(summary.trace.entries[1].outcome, "denied");
    assert!(summary.trace.entries[1].initialized);
    assert!(!rendered.contains("private resource content"));
    assert!(!rendered.contains("sk-secret"));
}

#[test]
fn mcp_server_trace_records_limit_and_closed_rejections() {
    let mut session = McpServerSession::new(
        empty_fixture(),
        McpServerSessionConfig::new(64 * 1024, Some(0)),
    );

    session.handle_frame(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#);
    session.handle_frame(r#"{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}"#);
    let summary = session.summary();

    assert_eq!(summary.trace.total_entries, 2);
    assert_eq!(summary.trace.error_entries, 2);
    assert_eq!(summary.trace.entries[0].outcome, "error:frame_limit");
    assert_eq!(summary.trace.entries[1].outcome, "error:closed");
}
