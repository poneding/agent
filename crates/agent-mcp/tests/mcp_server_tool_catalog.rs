use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use agent_mcp::{
    McpError, McpServerProfileRuntime, McpServerToolExportCatalog,
    build_mcp_server_tool_profile_from_manifest, parse_mcp_server_export_manifest_json,
};
use agent_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use agent_tool::{Tool, ToolDefinition, ToolInput, ToolName, ToolOutput};
use agent_types::{ActorRef, CorrelationId};
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
            description: format!("{} catalog fixture", self.name.as_str()),
            input_schema: json!({"type": "object"}),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, agent_tool::ToolError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(ToolOutput { value: input.value })
    }
}

fn expect_catalog_error(result: Result<McpServerProfileRuntime, McpError>) -> McpError {
    match result {
        Ok(_) => panic!("expected catalog error"),
        Err(error) => error,
    }
}

fn allow_policy(tool_name: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Execute,
        ResourceScope::new(ResourceKind::Tool, tool_name).expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    )
}

fn deny_policy(tool_name: &str) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Execute,
        ResourceScope::new(ResourceKind::Tool, tool_name).expect("scope"),
        PolicyDecision::Deny {
            reason: "fixture deny".to_string(),
        },
    )
}

#[test]
fn mcp_server_tool_catalog_builds_enabled_tool_profile() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut catalog = McpServerToolExportCatalog::default();
    catalog
        .register(CountingTool::new("test.echo", count.clone()))
        .expect("register catalog tool");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "test.echo"}]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime = build_mcp_server_tool_profile_from_manifest(&manifest, None, &catalog)
        .expect("build profile");
    let output = runtime
        .tool_exports()
        .call_tool(
            runtime.tools(),
            &allow_policy("test.echo"),
            ActorRef::system(),
            CorrelationId::new(),
            "test.echo",
            json!({"ok": true}),
        )
        .expect("call tool");

    assert_eq!(runtime.profile().tool_names, ["test.echo"]);
    assert_eq!(runtime.tool_exports().list_tools()[0].name, "test.echo");
    assert_eq!(output.output_json["ok"], true);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn mcp_server_tool_catalog_hides_disabled_tools() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut catalog = McpServerToolExportCatalog::default();
    catalog
        .register(CountingTool::new("test.hidden", count))
        .expect("register catalog tool");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "test.hidden", "enabled": false}]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime = build_mcp_server_tool_profile_from_manifest(&manifest, None, &catalog)
        .expect("build profile");

    assert!(runtime.profile().tool_names.is_empty());
    assert!(runtime.tool_exports().list_tools().is_empty());
    assert_eq!(runtime.tools().definitions().len(), 1);
}

#[test]
fn mcp_server_tool_catalog_rejects_unknown_tools_without_startup() {
    let catalog = McpServerToolExportCatalog::default();
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "test.missing"}]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let error = expect_catalog_error(build_mcp_server_tool_profile_from_manifest(
        &manifest, None, &catalog,
    ));

    assert!(error.to_string().contains("not found"));
    assert!(error.to_string().contains("test.missing"));
}

#[test]
fn mcp_server_tool_catalog_preserves_policy_gated_execution() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut catalog = McpServerToolExportCatalog::default();
    catalog
        .register(CountingTool::new("test.echo", count.clone()))
        .expect("register catalog tool");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "test.echo"}]
            }
          ]
        }"#,
    )
    .expect("manifest");

    let runtime = build_mcp_server_tool_profile_from_manifest(&manifest, None, &catalog)
        .expect("build profile");
    let error = runtime
        .tool_exports()
        .call_tool(
            runtime.tools(),
            &deny_policy("test.echo"),
            ActorRef::system(),
            CorrelationId::new(),
            "test.echo",
            json!({"api_key": "sk-secret"}),
        )
        .expect_err("denied call");

    assert!(error.to_string().contains("policy denied MCP tool export"));
    assert!(!error.to_string().contains("sk-secret"));
    assert_eq!(count.load(Ordering::SeqCst), 0);
}
