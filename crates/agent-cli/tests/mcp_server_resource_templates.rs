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

fn resource_template_profile_fixture() -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "template resource").expect("resource");
    let profile_path = temp.path().join("mcp-resource-templates.json");
    std::fs::write(
        &profile_path,
        r#"{
          "schema_version": 1,
          "profiles": [
            {
              "name": "default",
              "resource_templates": [
                {
                  "uri_template": "agent://workspace/file/{path}",
                  "name": "Workspace file",
                  "description": "Read a workspace file",
                  "path_template": "{path}",
                  "mime_type": "text/plain"
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
fn mcp_server_resource_templates_io_lists_template_descriptors() {
    let (_temp, profile_path) = resource_template_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"resources/templates/list","params":{}}"#,
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
    .expect("resource template stdio");
    let frames = parse_lines(&output);

    assert_eq!(
        frames[1]["result"]["resourceTemplates"][0]["uriTemplate"],
        "agent://workspace/file/{path}"
    );
    assert_eq!(
        frames[1]["result"]["resourceTemplates"][0]["description"],
        "Read a workspace file"
    );
    assert_eq!(
        frames[1]["result"]["resourceTemplates"][0]["mimeType"],
        "text/plain"
    );
}

#[test]
fn mcp_server_resource_templates_io_reads_template_resource() {
    let (_temp, profile_path) = resource_template_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"agent://workspace/file/README.md"}}"#,
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
    .expect("resource template stdio");
    let frames = parse_lines(&output);

    assert_eq!(
        frames[1]["result"]["uri"],
        "agent://workspace/file/README.md"
    );
    assert_eq!(frames[1]["result"]["contents"], "template resource");
    assert_eq!(frames[1]["result"]["mime_type"], "text/plain");
}

#[test]
fn mcp_server_resource_templates_io_rejects_invalid_template_reads() {
    let (_temp, profile_path) = resource_template_profile_fixture();
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"agent://workspace/file/../outside.txt"}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"resources/read","params":{"uri":"agent://workspace/missing/README.md"}}"#,
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
    .expect("resource template stdio");
    let frames = parse_lines(&output);

    assert_eq!(frames[1]["error"]["code"], -32602);
    assert!(
        frames[1]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("invalid MCP resource template path")
    );
    assert_eq!(frames[2]["error"]["code"], -32602);
    assert!(
        frames[2]["error"]["message"]
            .as_str()
            .expect("message")
            .contains("MCP resource export not found")
    );
}

#[test]
fn mcp_server_resource_templates_binary_outputs_protocol_frames_only() {
    let (_temp, profile_path) = resource_template_profile_fixture();
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
            r#"{{"jsonrpc":"2.0","id":2,"method":"resources/templates/list","params":{{}}}}"#
        )
        .expect("write templates");
    }

    let output = child.wait_with_output().expect("wait");
    let frames = parse_lines(&output.stdout);
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert!(output.status.success());
    assert_eq!(frames.len(), 2);
    assert_eq!(
        frames[1]["result"]["resourceTemplates"][0]["uriTemplate"],
        "agent://workspace/file/{path}"
    );
    assert_eq!(stderr, "");
}
