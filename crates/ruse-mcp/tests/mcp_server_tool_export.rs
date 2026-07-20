use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use ruse_mcp::{McpToolExportRegistry, exported_tool_from_definition};
use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_tool::{Tool, ToolDefinition, ToolInput, ToolName, ToolOutput, ToolRegistry};
use ruse_types::{ActorRef, CorrelationId};
use serde_json::json;

struct CountingTool {
    name: ToolName,
    count: Arc<AtomicUsize>,
    output: serde_json::Value,
}

impl CountingTool {
    fn new(name: &str, count: Arc<AtomicUsize>, output: serde_json::Value) -> Self {
        Self {
            name: ToolName::new(name).expect("valid tool name"),
            count,
            output,
        }
    }
}

impl Tool for CountingTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: format!("{} fixture tool", self.name.as_str()),
            input_schema: json!({
                "type": "object",
                "additionalProperties": true
            }),
        }
    }

    fn execute(&self, _input: ToolInput) -> Result<ToolOutput, ruse_tool::ToolError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(ToolOutput {
            value: self.output.clone(),
        })
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
fn mcp_server_tool_export_converts_definition_to_stable_descriptor() {
    let definition = CountingTool::new(
        "test.echo",
        Arc::new(AtomicUsize::new(0)),
        json!({"ok": true}),
    )
    .definition();

    let export = exported_tool_from_definition(definition, true).expect("export definition");
    let value = serde_json::to_value(&export).expect("export json");

    assert_eq!(value["name"], "test.echo");
    assert_eq!(value["description"], "test.echo fixture tool");
    assert_eq!(value["enabled"], true);
    assert_eq!(value["input_schema"]["type"], "object");
}

#[test]
fn mcp_server_tool_export_lists_enabled_exports_in_order() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut exports = McpToolExportRegistry::default();

    exports
        .export_tool(CountingTool::new("test.beta", count.clone(), json!({})).definition())
        .expect("export beta");
    exports
        .export_disabled_tool(CountingTool::new("test.alpha", count, json!({})).definition())
        .expect("export disabled alpha");
    exports
        .export_tool(
            CountingTool::new("test.gamma", Arc::new(AtomicUsize::new(0)), json!({})).definition(),
        )
        .expect("export gamma");

    let listed = exports.list_tools();

    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].name, "test.beta");
    assert_eq!(listed[1].name, "test.gamma");
}

#[test]
fn mcp_server_tool_export_rejects_duplicates() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut exports = McpToolExportRegistry::default();
    let definition = CountingTool::new("test.echo", count.clone(), json!({})).definition();

    exports
        .export_tool(definition.clone())
        .expect("first export succeeds");
    let error = exports
        .export_tool(definition)
        .expect_err("duplicate export fails");

    assert!(error.to_string().contains("duplicate MCP tool export"));
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[test]
fn mcp_server_tool_export_denied_call_does_not_execute_tool() {
    let count = Arc::new(AtomicUsize::new(0));
    let tool = CountingTool::new("test.echo", count.clone(), json!({"content": "ok"}));
    let mut tools = ToolRegistry::default();
    let mut exports = McpToolExportRegistry::default();

    exports.export_tool(tool.definition()).expect("export tool");
    tools.register(tool).expect("register tool");

    let error = exports
        .call_tool(
            &tools,
            &deny_policy("test.echo"),
            ActorRef::system(),
            CorrelationId::new(),
            "test.echo",
            json!({"secret": "sk-secret"}),
        )
        .expect_err("denied tool call fails");

    assert!(error.to_string().contains("policy denied MCP tool export"));
    assert_eq!(count.load(Ordering::SeqCst), 0);
    assert!(!error.to_string().contains("sk-secret"));
}

#[test]
fn mcp_server_tool_export_allowed_call_executes_and_bounds_output() {
    let count = Arc::new(AtomicUsize::new(0));
    let tool = CountingTool::new(
        "test.echo",
        count.clone(),
        json!({"content": "abcdefghijklmnopqrstuvwxyz"}),
    );
    let mut tools = ToolRegistry::default();
    let mut exports = McpToolExportRegistry::default().with_max_response_bytes(16);

    exports.export_tool(tool.definition()).expect("export tool");
    tools.register(tool).expect("register tool");

    let output = exports
        .call_tool(
            &tools,
            &allow_policy("test.echo"),
            ActorRef::system(),
            CorrelationId::new(),
            "test.echo",
            json!({"message": "hello"}),
        )
        .expect("allowed tool call");

    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert_eq!(output.output_json["truncated"], true);
    assert!(output.output_json["content"].as_str().unwrap().len() <= 16);
}
