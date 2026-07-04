use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use agent_mcp::{
    McpGateway, McpServerConfig, McpServerId, McpServerStatus, McpStdioClient, McpTransportConfig,
};
use agent_tool::{ToolInput, ToolName, ToolRegistry};
use serde_json::json;
use tempfile::TempDir;

fn compile_helper(temp: &TempDir) -> PathBuf {
    let source = temp.path().join("mcp_stdio_tool_call_helper.rs");
    fs::write(
        &source,
        r##"
use std::env;
use std::io::{self, BufRead, Write};
use std::process;

fn main() {
    match env::args().nth(1).as_deref() {
        Some("serve") => serve(),
        _ => process::exit(64),
    }
}

fn serve() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let mut stdout = io::stdout().lock();

    read_expected(&mut lines, "initialize");
    write_response(
        &mut stdout,
        br#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","serverInfo":{"name":"process-tool-server","version":"0.1.0"},"capabilities":{"tools":{}}}}
"#,
    );

    read_expected(&mut lines, "tools/list");
    write_tools(&mut stdout, 2);

    read_expected(&mut lines, "tools/list");
    write_tools(&mut stdout, 3);

    let call = read_expected(&mut lines, "tools/call");
    if !call.contains("\"name\":\"read_file\"") || !call.contains("\"path\":\"README.md\"") {
        process::exit(66);
    }
    write_response(
        &mut stdout,
        br#"{"jsonrpc":"2.0","id":4,"result":{"output_json":{"content":"remote read README.md"}}}
"#,
    );
}

fn read_expected<I>(lines: &mut I, method: &str) -> String
where
    I: Iterator<Item = io::Result<String>>,
{
    let line = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(_)) => process::exit(65),
        None => process::exit(65),
    };
    if !line.contains(&format!("\"method\":\"{method}\"")) {
        process::exit(66);
    }
    line
}

fn write_tools(stdout: &mut impl Write, id: i64) {
    let response = format!(
        r#"{{"jsonrpc":"2.0","id":{id},"result":{{"tools":[{{"name":"read_file","title":"Read file","description":"Read a file through stdio MCP","inputSchema":{{"type":"object","properties":{{"path":{{"type":"string"}}}}}}}}]}}}}
"#
    );
    write_response(stdout, response.as_bytes());
}

fn write_response(stdout: &mut impl Write, response: &[u8]) {
    stdout.write_all(response).expect("stdout");
    stdout.flush().expect("flush");
}
"##,
    )
    .expect("write helper source");

    let binary = temp.path().join(format!(
        "mcp-stdio-tool-call-helper{}",
        std::env::consts::EXE_SUFFIX
    ));
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let status = Command::new(rustc)
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .status()
        .expect("compile helper");
    assert!(status.success(), "helper program should compile");
    binary
}

fn server_id() -> McpServerId {
    McpServerId::new("stdio-tool-process").expect("server id")
}

fn stdio_config(command: &Path) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "Stdio Tool Process",
        McpTransportConfig::Stdio {
            command: command.display().to_string(),
            args: vec!["serve".to_string()],
            env: BTreeMap::new(),
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

#[tokio::test]
async fn mcp_client_stdio_tool_call_process_executes_gateway_registered_tool() {
    let temp = tempfile::tempdir().expect("tempdir");
    let helper = compile_helper(&temp);
    let config = stdio_config(&helper);
    let client = Arc::new(
        McpStdioClient::spawn(&config)
            .await
            .expect("spawn stdio client"),
    );
    let gateway = McpGateway::new(Arc::clone(&client));

    let report = gateway.discover_tools(std::slice::from_ref(&config)).await;

    assert_eq!(report.servers.len(), 1);
    assert_eq!(report.servers[0].status, McpServerStatus::Healthy);
    assert_eq!(report.tools.len(), 1);
    assert_eq!(
        report.tools[0].local_name,
        "mcp.stdio-tool-process.read_file"
    );
    assert!(report.diagnostics.is_empty());

    let mut registry = ToolRegistry::default();
    gateway
        .register_tools(&mut registry, &report)
        .expect("register tools");
    let tool = registry
        .get(&ToolName::new("mcp.stdio-tool-process.read_file").expect("tool name"))
        .expect("registered tool");
    let handle = tokio::runtime::Handle::current();
    let output = tokio::task::spawn_blocking(move || {
        let _guard = handle.enter();
        tool.execute(ToolInput {
            value: json!({
                "path": "README.md",
            }),
        })
    })
    .await
    .expect("tool execution task")
    .expect("tool execute");

    assert_eq!(output.value["content"], "remote read README.md");

    client.close().await.expect("close");
}
