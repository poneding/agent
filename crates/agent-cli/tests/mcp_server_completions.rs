use std::io::Cursor;

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

fn completion_profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "hello").expect("resource");
    let profile_path = temp.path().join("mcp-completions.json");
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
                  "arguments": [
                    {"name": "topic", "description": "Review topic", "required": true}
                  ],
                  "argument_completions": [
                    {"argument": "topic", "values": ["workspace diff", "worktree status"]}
                  ],
                  "messages": [
                    {"role": "user", "text": "Review {{topic}}."}
                  ]
                }
              ],
              "resource_templates": [
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file",
                  "path_template": "{path}",
                  "variable_completions": [
                    {"variable": "path", "values": ["README.md", "src/lib.rs"]}
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
fn mcp_server_completions_io_serves_prompt_argument_completion() {
    let (_temp, profile_path) = completion_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"completion/complete","params":{"ref":{"type":"ref/prompt","name":"review"},"argument":{"name":"topic","value":"work"}}}"#,
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
    .expect("completion stdio");
    let frames = parse_lines(&output);

    assert_eq!(
        frames[1]["result"]["completion"]["values"],
        serde_json::json!(["workspace diff", "worktree status"])
    );
    assert_eq!(frames[1]["result"]["completion"]["total"], 2);
    assert_eq!(frames[1]["result"]["completion"]["hasMore"], false);
}

#[test]
fn mcp_server_completions_io_serves_resource_template_completion() {
    let (_temp, profile_path) = completion_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"completion/complete","params":{"ref":{"type":"ref/resource","uri":"agent://workspace/file/{path}"},"argument":{"name":"path","value":"READ"}}}"#,
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
    .expect("completion stdio");
    let frames = parse_lines(&output);

    assert_eq!(
        frames[1]["result"]["completion"]["values"],
        serde_json::json!(["README.md"])
    );
}

#[test]
fn mcp_server_completions_io_requires_initialize_and_valid_params() {
    let (_temp, profile_path) = completion_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"completion/complete","params":{"ref":{"type":"ref/prompt","name":"review"},"argument":{"name":"topic","value":"work"}}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"completion/complete","params":{"ref":{"type":"ref/prompt","name":"review"},"argument":{"name":"missing","value":""}}}"#,
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
    .expect("completion stdio");
    let frames = parse_lines(&output);

    assert_eq!(frames[0]["error"]["code"], -32600);
    assert_eq!(frames[2]["error"]["code"], -32602);
    assert!(
        frames[2]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("unknown MCP prompt argument")
    );
}
