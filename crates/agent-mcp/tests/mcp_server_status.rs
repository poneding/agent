use agent_mcp::{
    MCP_SERVER_PROTOCOL_VERSION, McpResourceExportRegistry, McpServerCapabilities, McpServerInfo,
    McpServerState, McpToolExportRegistry, mcp_server_status_summary,
};
use agent_tool::{ToolDefinition, ToolName};
use serde_json::json;

fn state() -> McpServerState {
    McpServerState::new(
        McpServerInfo::new("agent", "0.1.0", MCP_SERVER_PROTOCOL_VERSION).expect("info"),
        McpServerCapabilities {
            tools: true,
            prompts: false,
            resources: true,
        },
    )
}

#[test]
fn mcp_server_status_summary_reports_counts_and_capabilities() {
    let mut tools = McpToolExportRegistry::default();
    tools
        .export_tool(ToolDefinition {
            name: ToolName::new("test.echo").expect("tool name"),
            description: "fixture".to_string(),
            input_schema: json!({"type": "object"}),
        })
        .expect("export tool");
    let mut resources = McpResourceExportRegistry::default();
    resources
        .export_resource("agent://readme", "README", None::<String>, "hello")
        .expect("export resource");

    let summary = mcp_server_status_summary(&state().initialized(), &tools, &resources);

    assert!(summary.available);
    assert!(summary.initialized);
    assert!(summary.capabilities.tools);
    assert!(!summary.capabilities.prompts);
    assert!(summary.capabilities.resources);
    assert!(summary.stdio_fixture_available);
    assert_eq!(summary.exported_tool_count, 1);
    assert_eq!(summary.exported_resource_count, 1);
    assert_eq!(summary.exported_tool_names, ["test.echo"]);
    assert_eq!(summary.exported_resource_uris, ["agent://readme"]);
}

#[test]
fn mcp_server_status_summary_redacts_secret_like_export_names() {
    let mut tools = McpToolExportRegistry::default();
    tools
        .export_tool(ToolDefinition {
            name: ToolName::new("secret.token").expect("tool name"),
            description: "fixture".to_string(),
            input_schema: json!({"type": "object"}),
        })
        .expect("export tool");

    let summary =
        mcp_server_status_summary(&state(), &tools, &McpResourceExportRegistry::default());
    let rendered = serde_json::to_string(&summary).expect("summary json");

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("secret.token"));
}
