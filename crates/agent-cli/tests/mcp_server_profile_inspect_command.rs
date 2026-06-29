use agent_cli::{CliCommand, McpServerCommand, parse_cli_invocation, render_cli_invocation};
use serde_json::Value;
use tempfile::TempDir;

fn profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("README.md"),
        "private fixture contents must not leak",
    )
    .expect("resource");
    let profile_path = temp.path().join("mcp-profile.json");
    std::fs::write(
        &profile_path,
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "tools": [{"name": "workspace.read_file"}],
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
fn mcp_server_profile_inspect_command_parses_flags() {
    let invocation = parse_cli_invocation([
        "mcp",
        "server",
        "profile",
        "inspect",
        "--profile",
        "profile.json",
        "--profile-name",
        "default",
        "--workspace",
        ".",
    ])
    .expect("parse profile inspect");

    assert_eq!(
        invocation.command,
        CliCommand::McpServer(McpServerCommand::ProfileInspect {
            profile_path: "profile.json".into(),
            profile_name: Some("default".to_string()),
            workspace_root: Some(".".into())
        })
    );
    assert!(
        parse_cli_invocation([
            "mcp",
            "server",
            "profile",
            "inspect",
            "--profile-name",
            "default"
        ])
        .is_err()
    );
}

#[test]
fn mcp_server_profile_inspect_command_renders_text_summary() {
    let (_temp, profile_path) = profile_fixture();
    let invocation = parse_cli_invocation([
        "mcp",
        "server",
        "profile",
        "inspect",
        "--profile",
        profile_path.to_str().expect("utf8 path"),
    ])
    .expect("parse profile inspect");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("MCP server profile: default"));
    assert!(output.stdout.contains("tools: 1"));
    assert!(output.stdout.contains("workspace.read_file"));
    assert!(output.stdout.contains("resources: 1"));
    assert!(output.stdout.contains("agent://readme"));
    assert!(
        output
            .stdout
            .contains("diagnostic error workspace_required")
    );
    assert!(output.stdout.contains("--workspace"));
    assert!(!output.stdout.contains("private fixture contents"));
}

#[test]
fn mcp_server_profile_inspect_command_renders_json_summary() {
    let (temp, profile_path) = profile_fixture();
    let invocation = parse_cli_invocation([
        "--json",
        "mcp",
        "server",
        "profile",
        "inspect",
        "--profile",
        profile_path.to_str().expect("utf8 path"),
        "--workspace",
        temp.path().to_str().expect("utf8 workspace"),
    ])
    .expect("parse profile inspect");

    let output = render_cli_invocation(invocation);
    let value: Value = serde_json::from_str(output.stdout.trim()).expect("json frame");

    assert_eq!(output.status_code, 0);
    assert_eq!(value["type"], "mcp_server_profile_inspection");
    assert_eq!(value["data"]["profile_name"], "default");
    assert_eq!(value["data"]["tools"][0]["name"], "workspace.read_file");
    assert_eq!(value["data"]["resources"][0]["uri"], "agent://readme");
    assert_eq!(value["data"]["resources"][0]["path"], "README.md");
    assert_eq!(value["data"]["resources"][0]["mime_type"], "text/markdown");
    assert_eq!(
        value["data"]["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .len(),
        0
    );
    assert!(!output.stdout.contains("private fixture contents"));
}
