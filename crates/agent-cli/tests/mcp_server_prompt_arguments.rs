use std::io::Cursor;
use std::io::Write;
use std::process::{Command, Stdio};

use agent_cli::{McpServerStdioConfig, run_mcp_server_stdio_io};
use serde_json::Value;
use tempfile::TempDir;

fn parse_lines(output: &[u8]) -> Vec<Value> {
    let stdout = String::from_utf8(output.to_vec()).expect("utf8 stdout");
    stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("json rpc line"))
        .collect()
}

fn prompt_argument_profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
    let profile_path = temp.path().join("mcp-prompt-arguments.json");
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
                  "arguments": [
                    {"name": "topic", "description": "Review topic", "required": true},
                    {"name": "tone", "description": "Optional tone", "required": false}
                  ],
                  "messages": [
                    {"role": "system", "text": "Use a {{tone}} tone."},
                    {"role": "user", "text": "Review {{topic}}."}
                  ]
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
fn mcp_server_prompt_arguments_io_lists_argument_descriptors() {
    let (_temp, profile_path) = prompt_argument_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"prompts/list","params":{}}"#,
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

    assert_eq!(
        frames[1]["result"]["prompts"][0]["arguments"][0]["name"],
        "topic"
    );
    assert_eq!(
        frames[1]["result"]["prompts"][0]["arguments"][0]["description"],
        "Review topic"
    );
    assert_eq!(
        frames[1]["result"]["prompts"][0]["arguments"][0]["required"],
        true
    );
    assert_eq!(
        frames[1]["result"]["prompts"][0]["arguments"][1]["name"],
        "tone"
    );
}

#[test]
fn mcp_server_prompt_arguments_io_gets_rendered_prompt() {
    let (_temp, profile_path) = prompt_argument_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"prompts/get","params":{"name":"review","arguments":{"topic":"workspace diff","tone":"concise"}}}"#,
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

    assert_eq!(
        frames[1]["result"]["messages"][0]["content"]["text"],
        "Use a concise tone."
    );
    assert_eq!(
        frames[1]["result"]["messages"][1]["content"]["text"],
        "Review workspace diff."
    );
}

#[test]
fn mcp_server_prompt_arguments_io_rejects_invalid_arguments() {
    let (_temp, profile_path) = prompt_argument_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"prompts/get","params":{"name":"review","arguments":{}}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"prompts/get","params":{"name":"review","arguments":{"topic":"workspace diff","extra":"unexpected"}}}"#,
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

    assert_eq!(frames[1]["error"]["code"], -32602);
    assert!(
        frames[1]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("missing required MCP prompt argument")
    );
    assert_eq!(frames[2]["error"]["code"], -32602);
    assert!(
        frames[2]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("unknown MCP prompt argument")
    );
}

#[test]
fn mcp_server_prompt_arguments_binary_outputs_protocol_frames_only() {
    let (_temp, profile_path) = prompt_argument_profile_fixture();
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
            r#"{{"jsonrpc":"2.0","id":2,"method":"prompts/get","params":{{"name":"review","arguments":{{"topic":"workspace diff","tone":"concise"}}}}}}"#
        )
        .expect("write get");
    }

    let output = child.wait_with_output().expect("wait");
    let frames = parse_lines(&output.stdout);
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert!(output.status.success());
    assert_eq!(frames.len(), 2);
    assert_eq!(
        frames[1]["result"]["messages"][1]["content"]["text"],
        "Review workspace diff."
    );
    assert_eq!(stderr, "");
}
