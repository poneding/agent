use agent_mcp::{
    MCP_SERVER_PROTOCOL_VERSION, McpPromptExport, McpPromptExportRegistry, McpPromptMessage,
    McpResourceExportRegistry, McpServerCapabilities, McpServerFixture, McpServerInfo,
    McpServerMethod, McpServerState, McpToolExportRegistry, handle_mcp_server_frame,
};
use agent_policy::StaticPolicyEngine;
use agent_tool::ToolRegistry;
use serde_json::Value;

fn server_state() -> McpServerState {
    McpServerState::new(
        McpServerInfo::new("agent-fixture", "0.1.0", MCP_SERVER_PROTOCOL_VERSION)
            .expect("server info"),
        McpServerCapabilities {
            tools: true,
            prompts: true,
            resources: true,
        },
    )
}

fn response_value(frame: &str) -> Value {
    serde_json::from_str(frame.trim_end()).expect("json response")
}

fn prompt_export(name: &str, enabled: bool) -> McpPromptExport {
    McpPromptExport::new(
        name,
        Some(format!("{name} prompt")),
        vec![McpPromptMessage::text("user", format!("Run {name}")).expect("message")],
        enabled,
    )
    .expect("prompt export")
}

#[test]
fn mcp_server_prompt_protocol_parses_prompt_methods() {
    assert_eq!(
        McpServerMethod::parse("prompts/list").expect("list"),
        McpServerMethod::PromptsList
    );
    assert_eq!(
        McpServerMethod::parse("prompts/get").expect("get"),
        McpServerMethod::PromptsGet
    );
}

#[test]
fn mcp_server_prompt_protocol_registry_hides_disabled_prompts() {
    let mut registry = McpPromptExportRegistry::default();
    registry
        .export_prompt(prompt_export("default", true))
        .expect("enabled prompt");
    registry
        .export_prompt(prompt_export("hidden", false))
        .expect("disabled prompt");

    let listed = registry.list_prompts();
    let missing = registry.get_prompt("hidden").expect_err("disabled hidden");

    assert_eq!(listed.prompts.len(), 1);
    assert_eq!(listed.prompts[0].name, "default");
    assert!(missing.to_string().contains("MCP prompt export not found"));
}

#[test]
fn mcp_server_prompt_protocol_lists_registered_prompts() {
    let mut prompt_exports = McpPromptExportRegistry::default();
    prompt_exports
        .export_prompt(prompt_export("default", true))
        .expect("prompt");
    let mut server = McpServerFixture::new(
        server_state(),
        ToolRegistry::default(),
        McpToolExportRegistry::default(),
        McpResourceExportRegistry::default(),
        StaticPolicyEngine::default(),
    )
    .with_prompt_exports(prompt_exports);
    handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );

    let frame = handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":2,"method":"prompts/list","params":{}}"#,
    );
    let value = response_value(&frame);

    assert_eq!(value["id"], 2);
    assert_eq!(value["result"]["prompts"][0]["name"], "default");
    assert_eq!(
        value["result"]["prompts"][0]["description"],
        "default prompt"
    );
}

#[test]
fn mcp_server_prompt_protocol_gets_registered_prompt_messages() {
    let mut prompt_exports = McpPromptExportRegistry::default();
    prompt_exports
        .export_prompt(prompt_export("default", true))
        .expect("prompt");
    let mut server = McpServerFixture::new(
        server_state(),
        ToolRegistry::default(),
        McpToolExportRegistry::default(),
        McpResourceExportRegistry::default(),
        StaticPolicyEngine::default(),
    )
    .with_prompt_exports(prompt_exports);
    handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );

    let frame = handle_mcp_server_frame(
        &mut server,
        r#"{"jsonrpc":"2.0","id":2,"method":"prompts/get","params":{"name":"default"}}"#,
    );
    let value = response_value(&frame);

    assert_eq!(value["result"]["name"], "default");
    assert_eq!(value["result"]["messages"][0]["role"], "user");
    assert_eq!(value["result"]["messages"][0]["content"]["type"], "text");
    assert_eq!(
        value["result"]["messages"][0]["content"]["text"],
        "Run default"
    );
}
