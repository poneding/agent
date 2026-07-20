use std::io::Cursor;
use std::io::Write;
use std::process::{Command, Stdio};

use ruse_cli::{
    CliCommand, McpServerCommand, McpServerStdioConfig, OutputMode, parse_cli_invocation,
    run_mcp_server_stdio_io,
};
use serde_json::Value;

fn parse_lines(output: &[u8]) -> Vec<Value> {
    let stdout = String::from_utf8(output.to_vec()).expect("utf8 stdout");
    stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("json rpc line"))
        .collect()
}

#[test]
fn mcp_server_stdio_command_parses_default_and_frame_limit() {
    let default = parse_cli_invocation(["mcp", "server", "stdio"]).expect("parse stdio");
    assert_eq!(
        default.command,
        CliCommand::McpServer(McpServerCommand::Stdio {
            max_frames: None,
            profile_path: None,
            profile_name: None,
            workspace_root: None
        })
    );

    let limited =
        parse_cli_invocation(["mcp", "server", "stdio", "--max-frames", "2"]).expect("parse");
    assert_eq!(
        limited.command,
        CliCommand::McpServer(McpServerCommand::Stdio {
            max_frames: Some(2),
            profile_path: None,
            profile_name: None,
            workspace_root: None
        })
    );

    let json = parse_cli_invocation(["--json", "mcp", "server", "stdio"]).expect("parse json");
    assert_eq!(json.output_mode, OutputMode::Json);
}

#[test]
fn mcp_server_stdio_command_io_serves_initialize_and_list_until_eof() {
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        ]
        .join("\n")
            + "\n",
    );
    let mut output = Vec::new();

    let summary =
        run_mcp_server_stdio_io(input, &mut output, McpServerStdioConfig::new(None)).unwrap();
    let frames = parse_lines(&output);

    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0]["result"]["initialized"], true);
    assert_eq!(frames[1]["result"]["tools"], Value::Array(Vec::new()));
    assert_eq!(summary.frames_received, 2);
    assert_eq!(summary.responses_sent, 2);
    assert_eq!(summary.close_reason, Some("eof".to_string()));
}

#[test]
fn mcp_server_stdio_command_io_honors_frame_limit() {
    let input = Cursor::new(
        [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        ]
        .join("\n")
            + "\n",
    );
    let mut output = Vec::new();

    let summary =
        run_mcp_server_stdio_io(input, &mut output, McpServerStdioConfig::new(Some(1))).unwrap();
    let frames = parse_lines(&output);

    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0]["result"]["initialized"], true);
    assert_eq!(summary.frames_received, 1);
    assert_eq!(
        summary.close_reason,
        Some("frame limit reached".to_string())
    );
}

#[test]
fn mcp_server_stdio_command_binary_smoke_outputs_protocol_frames_only() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .args(["mcp", "server", "stdio", "--max-frames", "2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ruse-cli");
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
        .expect("write list");
    }

    let output = child.wait_with_output().expect("wait");
    let frames = parse_lines(&output.stdout);
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert!(output.status.success());
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0]["result"]["initialized"], true);
    assert_eq!(frames[1]["result"]["resources"], Value::Array(Vec::new()));
    assert_eq!(stderr, "");
}

#[test]
fn mcp_server_stdio_command_rejects_json_mode_without_protocol_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .args(["--json", "mcp", "server", "stdio"])
        .output()
        .expect("run ruse-cli");
    let stdout = String::from_utf8(output.stdout).expect("stdout");
    let stderr = String::from_utf8(output.stderr).expect("stderr");

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(stdout, "");
    assert!(stderr.contains("does not support --json"));
}
