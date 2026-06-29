use std::io::Cursor;

use agent_cli::{
    CliCommand, McpServerCommand, McpServerStdioConfig, parse_cli_invocation,
    run_mcp_server_stdio_io,
};
use serde_json::Value;
use tempfile::TempDir;

fn parse_lines(output: &[u8]) -> Vec<Value> {
    let stdout = String::from_utf8(output.to_vec()).expect("utf8 stdout");
    stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("json rpc line"))
        .collect()
}

fn tool_profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "hello from workspace\n").expect("readme");
    let profile_path = temp.path().join("mcp-profile.json");
    std::fs::write(
        &profile_path,
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
    .expect("profile");
    (temp, profile_path)
}

fn resource_profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "resource only").expect("resource");
    let profile_path = temp.path().join("mcp-profile.json");
    std::fs::write(
        &profile_path,
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resources": [
                {
                  "uri": "agent://readme",
                  "name": "README",
                  "path": "README.md",
                  "mime_type": "text/markdown"
                }
              ]
            }
          ]
        }"#,
    )
    .expect("profile");
    (temp, profile_path)
}

#[test]
fn mcp_server_workspace_tool_profile_parses_workspace_flag() {
    let invocation = parse_cli_invocation([
        "mcp",
        "server",
        "stdio",
        "--profile",
        "profile.json",
        "--workspace",
        ".",
    ])
    .expect("parse workspace profile stdio");

    assert_eq!(
        invocation.command,
        CliCommand::McpServer(McpServerCommand::Stdio {
            max_frames: None,
            profile_path: Some("profile.json".into()),
            profile_name: None,
            workspace_root: Some(".".into())
        })
    );
}

#[test]
fn mcp_server_workspace_tool_profile_requires_workspace_for_tool_manifest() {
    let (_temp, profile_path) = tool_profile_fixture();
    let input = Cursor::new(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    let mut output = Vec::new();

    let error = run_mcp_server_stdio_io(
        input,
        &mut output,
        McpServerStdioConfig::new(None).with_profile(profile_path, None),
    )
    .expect_err("tool profile requires workspace");

    assert!(output.is_empty());
    assert!(error.to_string().contains("--workspace"));
    assert!(error.to_string().contains("tool"));
}

#[test]
fn mcp_server_workspace_tool_profile_allows_resource_only_without_workspace() {
    let (_temp, profile_path) = resource_profile_fixture();
    let input = Cursor::new(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    let mut output = Vec::new();

    let summary = run_mcp_server_stdio_io(
        input,
        &mut output,
        McpServerStdioConfig::new(None).with_profile(profile_path, None),
    )
    .expect("resource-only profile starts without workspace");

    assert_eq!(summary.frames_received, 1);
    assert!(!output.is_empty());
}

#[test]
fn mcp_server_workspace_tool_profile_lists_and_calls_exported_read_file() {
    let (temp, profile_path) = tool_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"workspace.read_file","arguments":{"path":"README.md"}}}"#,
        ]
        .join("\n")
            + "\n",
    );
    let mut output = Vec::new();

    let summary = run_mcp_server_stdio_io(
        input,
        &mut output,
        McpServerStdioConfig::new(None)
            .with_profile(profile_path, None)
            .with_workspace(temp.path()),
    )
    .expect("tool profile stdio");
    let frames = parse_lines(&output);

    assert_eq!(summary.frames_received, 3);
    assert_eq!(
        frames[1]["result"]["tools"][0]["name"],
        "workspace.read_file"
    );
    assert_eq!(frames[2]["result"]["output_json"]["path"], "README.md");
    assert_eq!(
        frames[2]["result"]["output_json"]["content"],
        "hello from workspace\n"
    );
}

#[test]
fn mcp_server_workspace_tool_profile_rejects_non_exported_tool_call() {
    let (temp, profile_path) = tool_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"workspace.search_text","arguments":{"path":".","query":"hello"}}}"#,
        ]
        .join("\n")
            + "\n",
    );
    let mut output = Vec::new();

    run_mcp_server_stdio_io(
        input,
        &mut output,
        McpServerStdioConfig::new(None)
            .with_profile(profile_path, None)
            .with_workspace(temp.path()),
    )
    .expect("tool profile stdio");
    let frames = parse_lines(&output);

    assert_eq!(frames[1]["error"]["code"], -32602);
    assert!(
        frames[1]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("MCP tool export not found")
    );
}
