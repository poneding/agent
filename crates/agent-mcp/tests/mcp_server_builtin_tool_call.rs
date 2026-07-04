use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_mcp::{
    MCP_SERVER_PROTOCOL_VERSION, McpServerBuiltinToolExportCatalogConfig, McpServerCapabilities,
    McpServerFixture, McpServerInfo, McpServerState, build_mcp_server_builtin_tool_export_catalog,
    build_mcp_server_tool_profile_from_manifest, handle_mcp_server_frame,
    parse_mcp_server_export_manifest_json,
};
use agent_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use serde_json::Value;

fn unique_workspace(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-mcp-tool-call-{name}-{nanos}"))
}

fn response_value(frame: &str) -> Value {
    serde_json::from_str(frame.trim_end()).expect("json response")
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

#[test]
fn mcp_server_builtin_tool_call_lists_and_executes_read_file() {
    let root = unique_workspace("read-file");
    fs::create_dir_all(&root).expect("workspace dir");
    fs::write(root.join("README.md"), "hello direct mcp\n").expect("readme");
    let catalog = build_mcp_server_builtin_tool_export_catalog(
        McpServerBuiltinToolExportCatalogConfig::new(&root),
    )
    .expect("catalog");
    let manifest = parse_mcp_server_export_manifest_json(
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.read_file"}]
            }
          ]
        }"#,
    )
    .expect("manifest");
    let runtime =
        build_mcp_server_tool_profile_from_manifest(&manifest, None, &catalog).expect("profile");
    let (_profile, tools, tool_exports, _prompt_exports, resource_exports) = runtime.into_parts();
    let policy = StaticPolicyEngine::default().with_decision(
        CapabilityAction::Execute,
        ResourceScope::new(ResourceKind::Tool, "workspace.read_file").expect("scope"),
        PolicyDecision::Allow {
            reason: "fixture allow".to_string(),
        },
    );
    let mut server = McpServerFixture::new(
        server_state(),
        tools,
        tool_exports,
        resource_exports,
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
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"workspace.read_file","arguments":{"path":"README.md"}}}"#,
    ));

    assert_eq!(list["result"]["tools"][0]["name"], "workspace.read_file");
    assert_eq!(call["result"]["output_json"]["path"], "README.md");
    assert_eq!(
        call["result"]["output_json"]["content"],
        "hello direct mcp\n"
    );
}
