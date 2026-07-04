use std::process::Command;

use agent_cli::{
    CliCommand, McpServerCommand, McpServerCommandRuntime, OutputMode, parse_cli_invocation,
    render_cli_invocation, render_mcp_server_command,
};
use agent_mcp::{McpServerStatusCapabilities, McpServerStatusSummary};
use serde_json::Value;

fn summary() -> McpServerStatusSummary {
    McpServerStatusSummary {
        available: true,
        protocol_version: "2025-11-25".to_string(),
        server_name: "agent".to_string(),
        server_version: "0.1.0".to_string(),
        initialized: false,
        capabilities: McpServerStatusCapabilities {
            tools: true,
            prompts: false,
            resources: true,
        },
        stdio_fixture_available: true,
        exported_tool_count: 1,
        exported_resource_count: 1,
        exported_tool_names: vec!["test.echo".to_string()],
        exported_resource_uris: vec!["agent://readme".to_string()],
    }
}

#[test]
fn mcp_server_commands_parse_status_and_json_mode() {
    let invocation = parse_cli_invocation(["mcp", "server", "status"]).expect("parse status");
    assert_eq!(
        invocation.command,
        CliCommand::McpServer(McpServerCommand::Status)
    );

    let json = parse_cli_invocation(["--json", "mcp", "server", "status"]).expect("parse json");
    assert_eq!(json.output_mode, OutputMode::Json);
    assert_eq!(
        json.command,
        CliCommand::McpServer(McpServerCommand::Status)
    );
}

#[test]
fn mcp_server_commands_render_text_status_without_starting_server() {
    let mut runtime = McpServerCommandRuntime::new(summary());
    let output =
        render_mcp_server_command(&mut runtime, McpServerCommand::Status, OutputMode::Text);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("MCP server status: available"));
    assert!(output.stdout.contains("health=degraded"));
    assert!(output.stdout.contains("stdio_fixture=available"));
    assert!(output.stdout.contains("exported_tools=1 test.echo"));
    assert!(
        output
            .stdout
            .contains("exported_resources=1 agent://readme")
    );
}

#[test]
fn mcp_server_commands_render_json_status() {
    let mut runtime = McpServerCommandRuntime::new(summary());
    let output =
        render_mcp_server_command(&mut runtime, McpServerCommand::Status, OutputMode::Json);
    let value: Value = serde_json::from_str(output.stdout.trim()).expect("status json");

    assert_eq!(output.status_code, 0);
    assert_eq!(value["type"], "mcp_server_status");
    assert_eq!(value["data"]["capabilities"]["tools"], true);
    assert_eq!(value["data"]["exported_tool_names"][0], "test.echo");
    assert_eq!(value["data"]["health"]["status"], "degraded");
    assert_eq!(
        value["data"]["health"]["diagnostics"][0]["code"],
        "mcp.server.not_initialized"
    );
}

#[test]
fn mcp_server_commands_render_json_health_for_capability_mismatch() {
    let mut summary = summary();
    summary.initialized = true;
    summary.capabilities.tools = false;
    summary.capabilities.resources = false;
    let mut runtime = McpServerCommandRuntime::new(summary);

    let output =
        render_mcp_server_command(&mut runtime, McpServerCommand::Status, OutputMode::Json);
    let value: Value = serde_json::from_str(output.stdout.trim()).expect("status json");

    assert_eq!(output.status_code, 0);
    assert_eq!(value["data"]["health"]["status"], "unhealthy");
    assert!(
        value["data"]["health"]["next_actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action == "mcp.server.fix_capabilities")
    );
    assert!(
        value["data"]["health"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["code"] == "mcp.server.tool_exports_without_capability")
    );
}

#[test]
fn mcp_server_commands_default_json_invocation_is_parseable() {
    let output =
        render_cli_invocation(parse_cli_invocation(["--json", "mcp", "server", "status"]).unwrap());
    let value: Value = serde_json::from_str(output.stdout.trim()).expect("status json");

    assert_eq!(output.status_code, 0);
    assert_eq!(value["type"], "mcp_server_status");
    assert_eq!(value["data"]["available"], true);
    assert_eq!(value["data"]["exported_tool_count"], 0);
    assert_eq!(value["data"]["exported_resource_count"], 0);
    assert_eq!(value["data"]["health"]["status"], "degraded");
}

#[test]
fn mcp_server_commands_binary_smoke_status() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["mcp", "server", "status"])
        .output()
        .expect("run agent-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    assert!(stdout.contains("MCP server status: available"));
    assert!(stdout.contains("exported_tools=0 (none)"));
}
