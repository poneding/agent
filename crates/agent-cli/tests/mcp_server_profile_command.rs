use std::io::Cursor;
use std::io::Write;
use std::process::{Command, Stdio};

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

fn profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "profile resource").expect("resource");
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
fn mcp_server_profile_command_parses_profile_flags() {
    let invocation = parse_cli_invocation([
        "mcp",
        "server",
        "stdio",
        "--profile",
        "profile.json",
        "--profile-name",
        "default",
        "--max-frames",
        "3",
    ])
    .expect("parse profile stdio");

    assert_eq!(
        invocation.command,
        CliCommand::McpServer(McpServerCommand::Stdio {
            max_frames: Some(3),
            profile_path: Some("profile.json".into()),
            profile_name: Some("default".to_string())
        })
    );
    assert!(parse_cli_invocation(["mcp", "server", "stdio", "--profile-name", "default"]).is_err());
}

#[test]
fn mcp_server_profile_command_io_lists_manifest_resources() {
    let (_temp, profile_path) = profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{}}"#,
        ]
        .join("\n")
            + "\n",
    );
    let mut output = Vec::new();

    let summary = run_mcp_server_stdio_io(
        input,
        &mut output,
        McpServerStdioConfig::new(None).with_profile(profile_path, None),
    )
    .expect("stdio profile");
    let frames = parse_lines(&output);

    assert_eq!(summary.frames_received, 2);
    assert_eq!(frames[0]["result"]["initialized"], true);
    assert_eq!(frames[1]["result"]["resources"][0]["uri"], "agent://readme");
    assert_eq!(
        frames[1]["result"]["resources"][0]["mime_type"],
        "text/markdown"
    );
}

#[test]
fn mcp_server_profile_command_binary_outputs_protocol_frames_only() {
    let (_temp, profile_path) = profile_fixture();
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "mcp",
            "server",
            "stdio",
            "--profile",
            profile_path.to_str().expect("utf8 path"),
            "--max-frames",
            "2",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn agent-cli");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{}}}}"#
        )
        .expect("write init");
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":2,"method":"resources/list","params":{{}}}}"#
        )
        .expect("write resources");
    }

    let output = child.wait_with_output().expect("wait");
    let frames = parse_lines(&output.stdout);
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert!(output.status.success());
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[1]["result"]["resources"][0]["uri"], "agent://readme");
    assert_eq!(stderr, "");
}

#[test]
fn mcp_server_profile_command_invalid_manifest_fails_without_protocol_stdout() {
    let temp = TempDir::new().expect("tempdir");
    let profile_path = temp.path().join("profile.json");
    std::fs::write(&profile_path, "{not json api_key sk-secret").expect("profile");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "mcp",
            "server",
            "stdio",
            "--profile",
            profile_path.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("run agent-cli");
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout, "");
    assert!(stderr.contains("mcp server stdio failed"));
    assert!(!stderr.contains("sk-secret"));
}
