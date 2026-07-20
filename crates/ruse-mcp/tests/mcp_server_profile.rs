use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use ruse_mcp::{
    McpServerExportProfileBuilder, empty_mcp_server_export_profile, handle_mcp_server_frame,
};
use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_tool::{Tool, ToolDefinition, ToolInput, ToolName, ToolOutput};
use serde_json::json;

struct CountingTool {
    name: ToolName,
    count: Arc<AtomicUsize>,
}

impl CountingTool {
    fn new(name: &str, count: Arc<AtomicUsize>) -> Self {
        Self {
            name: ToolName::new(name).expect("tool name"),
            count,
        }
    }
}

impl Tool for CountingTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: format!("{} fixture", self.name.as_str()),
            input_schema: json!({"type": "object"}),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ruse_tool::ToolError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(ToolOutput { value: input.value })
    }
}

#[test]
fn mcp_server_profile_default_exports_nothing() {
    let runtime = empty_mcp_server_export_profile("default").expect("empty profile");

    assert_eq!(runtime.profile().name, "default");
    assert!(runtime.profile().tool_names.is_empty());
    assert!(runtime.profile().prompt_names.is_empty());
    assert!(runtime.profile().resource_uris.is_empty());
    assert!(runtime.tools().definitions().is_empty());
    assert!(runtime.tool_exports().list_tools().is_empty());
    assert!(runtime.prompt_exports().list_prompts().prompts.is_empty());
    assert!(runtime.resource_exports().list_resources().is_empty());
}

#[test]
fn mcp_server_profile_builds_deterministic_fixture_exports() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut builder = McpServerExportProfileBuilder::new("fixture").expect("builder");

    builder
        .export_resource("agent://readme", "README", Some("text/plain"), "hello")
        .expect("resource");
    builder
        .export_tool(CountingTool::new("test.echo", count.clone()))
        .expect("tool");
    let runtime = builder.build();

    assert_eq!(runtime.profile().tool_names, ["test.echo"]);
    assert!(runtime.profile().prompt_names.is_empty());
    assert_eq!(runtime.profile().resource_uris, ["agent://readme"]);
    assert_eq!(runtime.tools().definitions()[0].name.as_str(), "test.echo");
    assert_eq!(runtime.tool_exports().list_tools()[0].name, "test.echo");
    assert_eq!(
        runtime.resource_exports().list_resources()[0].uri,
        "agent://readme"
    );
}

#[test]
fn mcp_server_profile_omits_disabled_entries_from_visible_profile() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut builder = McpServerExportProfileBuilder::new("fixture").expect("builder");

    builder
        .export_disabled_tool(CountingTool::new("test.hidden", count))
        .expect("disabled tool");
    builder
        .export_disabled_resource("agent://hidden", "Hidden", None::<String>, "hidden")
        .expect("disabled resource");
    let runtime = builder.build();

    assert!(runtime.profile().tool_names.is_empty());
    assert!(runtime.profile().prompt_names.is_empty());
    assert!(runtime.profile().resource_uris.is_empty());
    assert!(runtime.tool_exports().list_tools().is_empty());
    assert!(runtime.resource_exports().list_resources().is_empty());
    assert_eq!(runtime.tools().definitions().len(), 1);
}

#[test]
fn mcp_server_profile_rejects_duplicates_and_secret_like_names() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut builder = McpServerExportProfileBuilder::new("fixture").expect("builder");

    builder
        .export_tool(CountingTool::new("test.echo", count.clone()))
        .expect("first tool");
    let duplicate = builder
        .export_tool(CountingTool::new("test.echo", count))
        .expect_err("duplicate");
    let secret_profile = match McpServerExportProfileBuilder::new("api_key sk-secret") {
        Ok(_) => panic!("secret profile should fail"),
        Err(error) => error,
    };

    assert!(
        duplicate
            .to_string()
            .contains("duplicate MCP server profile tool")
    );
    assert!(!secret_profile.to_string().contains("sk-secret"));
    assert!(secret_profile.to_string().contains("[redacted]"));
}

#[test]
fn mcp_server_profile_parts_can_back_mcp_server_fixture() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut builder = McpServerExportProfileBuilder::new("fixture").expect("builder");
    builder
        .export_tool(CountingTool::new("test.echo", count.clone()))
        .expect("tool");
    let (_profile, tools, tool_exports, _prompt_exports, resource_exports) =
        builder.build().into_parts();
    let policy = StaticPolicyEngine::default().with_decision(
        CapabilityAction::Execute,
        ResourceScope::new(ResourceKind::Tool, "test.echo").expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    );
    let mut fixture = ruse_mcp::McpServerFixture::new(
        ruse_mcp::McpServerState::new(
            ruse_mcp::McpServerInfo::new(
                "agent-profile",
                "0.1.0",
                ruse_mcp::MCP_SERVER_PROTOCOL_VERSION,
            )
            .expect("info"),
            ruse_mcp::McpServerCapabilities {
                tools: true,
                prompts: false,
                resources: true,
            },
        ),
        tools,
        tool_exports,
        resource_exports,
        policy,
    );

    handle_mcp_server_frame(
        &mut fixture,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );
    let response = handle_mcp_server_frame(
        &mut fixture,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"test.echo","arguments":{"ok":true}}}"#,
    );

    assert!(response.contains(r#""ok":true"#));
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
