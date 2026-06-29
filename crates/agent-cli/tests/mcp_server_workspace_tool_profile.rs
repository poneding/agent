use std::io::Cursor;

use agent_cli::{
    CliCommand, McpServerCommand, McpServerStdioConfig, parse_cli_invocation,
    run_mcp_server_stdio_io,
};
use tempfile::TempDir;

fn tool_profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
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
