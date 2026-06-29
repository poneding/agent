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

fn prompt_profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
    let profile_path = temp.path().join("mcp-prompt-profile.json");
    std::fs::write(
        &profile_path,
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "prompts": [
                {
                  "name": "review",
                  "description": "Review current changes",
                  "messages": [
                    {"role": "system", "text": "You review changes carefully."},
                    {"role": "user", "text": "Review the workspace diff."}
                  ]
                },
                {
                  "name": "hidden",
                  "description": "Hidden prompt",
                  "enabled": false,
                  "messages": [{"role": "user", "text": "Do not list me."}]
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
fn mcp_server_prompt_profile_parses_profile_flags() {
    let invocation = parse_cli_invocation([
        "mcp",
        "server",
        "stdio",
        "--profile",
        "prompt-profile.json",
        "--profile-name",
        "default",
    ])
    .expect("parse prompt profile stdio");

    assert_eq!(
        invocation.command,
        CliCommand::McpServer(McpServerCommand::Stdio {
            max_frames: None,
            profile_path: Some("prompt-profile.json".into()),
            profile_name: Some("default".to_string()),
            workspace_root: None
        })
    );
}

#[test]
fn mcp_server_prompt_profile_io_lists_manifest_prompts() {
    let (_temp, profile_path) = prompt_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"prompts/list","params":{}}"#,
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
    .expect("prompt profile stdio");
    let frames = parse_lines(&output);

    assert_eq!(summary.frames_received, 2);
    assert_eq!(frames[0]["result"]["capabilities"]["prompts"], true);
    assert_eq!(frames[1]["result"]["prompts"].as_array().unwrap().len(), 1);
    assert_eq!(frames[1]["result"]["prompts"][0]["name"], "review");
    assert_eq!(
        frames[1]["result"]["prompts"][0]["description"],
        "Review current changes"
    );
}

#[test]
fn mcp_server_prompt_profile_io_gets_manifest_prompt_messages() {
    let (_temp, profile_path) = prompt_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"prompts/get","params":{"name":"review"}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"prompts/get","params":{"name":"hidden"}}"#,
        ]
        .join("\n")
            + "\n",
    );
    let mut output = Vec::new();

    run_mcp_server_stdio_io(
        input,
        &mut output,
        McpServerStdioConfig::new(None).with_profile(profile_path, None),
    )
    .expect("prompt profile stdio");
    let frames = parse_lines(&output);

    assert_eq!(frames[1]["result"]["name"], "review");
    assert_eq!(frames[1]["result"]["messages"][0]["role"], "system");
    assert_eq!(
        frames[1]["result"]["messages"][1]["content"]["text"],
        "Review the workspace diff."
    );
    assert_eq!(frames[2]["error"]["code"], -32602);
    assert!(
        frames[2]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("MCP prompt export not found")
    );
}

#[test]
fn mcp_server_prompt_profile_binary_outputs_protocol_frames_only() {
    let (_temp, profile_path) = prompt_profile_fixture();
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "mcp",
            "server",
            "stdio",
            "--profile",
            profile_path.to_str().expect("utf8 path"),
            "--max-frames",
            "3",
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
            r#"{{"jsonrpc":"2.0","id":2,"method":"prompts/list","params":{{}}}}"#
        )
        .expect("write list");
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":3,"method":"prompts/get","params":{{"name":"review"}}}}"#
        )
        .expect("write get");
    }

    let output = child.wait_with_output().expect("wait");
    let frames = parse_lines(&output.stdout);
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert!(output.status.success());
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[1]["result"]["prompts"][0]["name"], "review");
    assert_eq!(
        frames[2]["result"]["messages"][0]["content"]["type"],
        "text"
    );
    assert_eq!(stderr, "");
}
