use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use ruse_mcp::{
    MCP_SERVER_PROTOCOL_VERSION, McpResourceExportRegistry, McpServerCapabilities,
    McpServerFixture, McpServerInfo, McpServerState, McpToolExportRegistry,
    handle_mcp_server_frame,
};
use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_tool::{Tool, ToolDefinition, ToolInput, ToolName, ToolOutput, ToolRegistry};
use serde_json::{Value, json};

struct CountingTool {
    name: ToolName,
    count: Arc<AtomicUsize>,
}

impl CountingTool {
    fn new(name: &str, count: Arc<AtomicUsize>) -> Self {
        Self {
            name: ToolName::new(name).expect("valid name"),
            count,
        }
    }
}

impl Tool for CountingTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: "counting fixture tool".to_string(),
            input_schema: json!({
                "type": "object",
                "additionalProperties": true
            }),
        }
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ruse_tool::ToolError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        Ok(ToolOutput { value: input.value })
    }
}

fn server_state() -> McpServerState {
    McpServerState::new(
        McpServerInfo::new("agent-fixture", "0.1.0", MCP_SERVER_PROTOCOL_VERSION)
            .expect("server info"),
        McpServerCapabilities {
            tools: true,
            prompts: false,
            resources: true,
        },
    )
}

fn response_value(frame: &str) -> Value {
    serde_json::from_str(frame.trim_end()).expect("json response")
}

#[test]
fn mcp_server_stdio_initialize_sets_state_and_renders_success_frame() {
    let mut server = McpServerFixture::new(
        server_state(),
        ToolRegistry::default(),
        McpToolExportRegistry::default(),
        McpResourceExportRegistry::default(),
        StaticPolicyEngine::default(),
    );

    let frame = handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":"init-1","method":"initialize","params":{}}"#,
    );
    let value = response_value(&frame);

    assert!(frame.ends_with('\n'));
    assert!(server.state().initialized);
    assert_eq!(value["id"], "init-1");
    assert_eq!(value["result"]["initialized"], true);
    assert_eq!(value["result"]["server_info"]["name"], "agent-fixture");
    assert_eq!(
        value["result"]["protocol_version"],
        MCP_SERVER_PROTOCOL_VERSION
    );
}

#[test]
fn mcp_server_stdio_lists_and_calls_policy_allowed_tools() {
    let count = Arc::new(AtomicUsize::new(0));
    let tool = CountingTool::new("test.echo", count.clone());
    let mut tools = ToolRegistry::default();
    let mut exports = McpToolExportRegistry::default();
    let policy = StaticPolicyEngine::default().with_decision(
        CapabilityAction::Execute,
        ResourceScope::new(ResourceKind::Tool, "test.echo").expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    );

    exports.export_tool(tool.definition()).expect("export tool");
    tools.register(tool).expect("register tool");

    let mut server = McpServerFixture::new(
        server_state(),
        tools,
        exports,
        McpResourceExportRegistry::default(),
        policy,
    );
    handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );

    let list = response_value(&handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    ));
    let call = response_value(&handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"test.echo","arguments":{"message":"hello"}}}"#,
    ));

    assert_eq!(list["result"]["tools"][0]["name"], "test.echo");
    assert_eq!(call["result"]["output_json"]["message"], "hello");
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn mcp_server_stdio_lists_and_reads_policy_allowed_resources() {
    let mut resources = McpResourceExportRegistry::default().with_max_content_bytes(5);
    resources
        .export_resource(
            "agent://readme",
            "README",
            Some("text/plain"),
            "hello world",
        )
        .expect("export resource");
    let policy = StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::File, "agent://readme").expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    );
    let mut server = McpServerFixture::new(
        server_state(),
        ToolRegistry::default(),
        McpToolExportRegistry::default(),
        resources,
        policy,
    );
    handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );

    let list = response_value(&handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}"#,
    ));
    let read = response_value(&handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"agent://readme"}}"#,
    ));

    assert_eq!(list["result"]["resources"][0]["uri"], "agent://readme");
    assert_eq!(read["result"]["contents"], "hello");
    assert_eq!(read["result"]["truncated"], true);
}

#[test]
fn mcp_server_stdio_malformed_frame_preserves_initialized_state() {
    let mut server = McpServerFixture::new(
        server_state(),
        ToolRegistry::default(),
        McpToolExportRegistry::default(),
        McpResourceExportRegistry::default(),
        StaticPolicyEngine::default(),
    );
    handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );

    let error = response_value(&handle_mcp_server_frame(&mut server, "{not json"));

    assert!(server.state().initialized);
    assert_eq!(error["id"], Value::Null);
    assert_eq!(error["error"]["code"], -32700);
    assert_eq!(error["error"]["message"], "invalid JSON-RPC frame");
}

#[test]
fn mcp_server_stdio_rejects_unknown_method_and_continues_serving() {
    let mut server = McpServerFixture::new(
        server_state(),
        ToolRegistry::default(),
        McpToolExportRegistry::default(),
        McpResourceExportRegistry::default(),
        StaticPolicyEngine::default(),
    );
    handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );

    let unknown = response_value(&handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":2,"method":"missing/method","params":{}}"#,
    ));
    let list = response_value(&handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}"#,
    ));

    assert_eq!(unknown["error"]["code"], -32601);
    assert_eq!(list["result"]["tools"], json!([]));
}
