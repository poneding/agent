use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ruse_mcp::{
    FixtureMcpClientStdioTransport, McpClient as _, McpServerConfig, McpServerId, McpServerStatus,
    McpStdioClient, McpTransportConfig, check_mcp_server_health,
};
use serde_json::json;
use tempfile::TempDir;

fn server_id() -> McpServerId {
    McpServerId::new("stdio-client").expect("server id")
}

fn stdio_config(command: impl Into<String>) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "Stdio Client",
        McpTransportConfig::Stdio {
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

fn fixture_config() -> McpServerConfig {
    stdio_config("mcp-fixture")
}

fn initialize_result() -> serde_json::Value {
    json!({
        "protocol_version": "2025-11-25",
        "server_info": {
            "name": "fixture-server",
            "version": "1.2.3"
        },
        "capabilities": {
            "tools": true,
            "prompts": false,
            "resources": {}
        },
        "initialized": true
    })
}

fn compile_helper(temp: &TempDir) -> PathBuf {
    let source = temp.path().join("mcp_stdio_client_helper.rs");
    fs::write(
        &source,
        r##"
use std::env;
use std::io::{self, BufRead, Write};
use std::process;

fn main() {
    match env::args().nth(1).as_deref() {
        Some("health") => {
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            if lines.next().transpose().expect("init").is_none() {
                process::exit(65);
            }
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(br#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-11-25","serverInfo":{"name":"process-server","version":"0.1.0"},"capabilities":{"tools":{}}}}
"#)
                .expect("init stdout");
            stdout.flush().expect("init flush");

            if lines.next().transpose().expect("tools").is_none() {
                process::exit(65);
            }
            stdout
                .write_all(br#"{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"test.echo","title":"Echo","description":"Echo input","inputSchema":{"type":"object"}}]}}
"#)
                .expect("tools stdout");
            stdout.flush().expect("tools flush");
        }
        _ => process::exit(64),
    }
}
"##,
    )
    .expect("write helper source");

    let binary = temp.path().join(format!(
        "mcp-stdio-client-helper{}",
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

fn process_config(command: &Path) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "Process Server",
        McpTransportConfig::Stdio {
            command: command.display().to_string(),
            args: vec!["health".to_string()],
            env: BTreeMap::new(),
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

#[tokio::test]
async fn mcp_client_stdio_mcp_client_maps_initialize_result() {
    let config = fixture_config();
    let transport = FixtureMcpClientStdioTransport::from_inbound_frames([json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": initialize_result()
    })
    .to_string()]);
    let client = McpStdioClient::new(transport);

    let init = client.initialize(&config).await.expect("initialize");

    assert_eq!(init.server_id, config.id);
    assert_eq!(init.protocol_version, "2025-11-25");
    assert_eq!(init.server_name, "fixture-server");
    assert_eq!(init.server_version.as_deref(), Some("1.2.3"));
    assert!(init.capabilities.tools);
    assert!(!init.capabilities.prompts);
    assert!(init.capabilities.resources);
}

#[tokio::test]
async fn mcp_client_stdio_mcp_client_maps_tools_list_result() {
    let config = fixture_config();
    let transport = FixtureMcpClientStdioTransport::from_inbound_frames([
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": initialize_result()
        })
        .to_string(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "tools": [
                    {
                        "name": "test.echo",
                        "title": "Echo",
                        "description": "Echo input",
                        "inputSchema": {
                            "type": "object"
                        }
                    }
                ]
            }
        })
        .to_string(),
    ]);
    let client = McpStdioClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    let tools = client.list_tools(&config.id).await.expect("tools");

    assert_eq!(tools.server_id, config.id);
    assert_eq!(tools.tools.len(), 1);
    assert_eq!(tools.tools[0].name, "test.echo");
    assert_eq!(tools.tools[0].title.as_deref(), Some("Echo"));
    assert_eq!(tools.tools[0].input_schema["type"], "object");
}

#[tokio::test]
async fn mcp_client_stdio_mcp_client_health_reports_healthy_for_process() {
    let temp = tempfile::tempdir().expect("tempdir");
    let helper = compile_helper(&temp);
    let config = process_config(&helper);
    let client = McpStdioClient::spawn(&config)
        .await
        .expect("spawn stdio client");

    let health = check_mcp_server_health(&client, &config).await;

    assert_eq!(health.status, McpServerStatus::Healthy);
    assert!(health.message.contains("process-server"));
    assert!(health.message.contains("1 tools"));
    client.close().await.expect("close");
}

#[tokio::test]
async fn mcp_client_stdio_mcp_client_invalid_initialize_shape_is_redacted() {
    let config = fixture_config();
    let secret_protocol = "sk-secret-token-should-not-leak";
    let transport = FixtureMcpClientStdioTransport::from_inbound_frames([json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "protocol_version": secret_protocol,
            "server_info": {
                "version": "1.2.3"
            },
            "capabilities": {
                "tools": true
            }
        }
    })
    .to_string()]);
    let client = McpStdioClient::new(transport);

    let error = client
        .initialize(&config)
        .await
        .expect_err("invalid initialize shape");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client stdio initialize result"));
    assert!(!message.contains(secret_protocol));
    assert!(!message.contains("sk-secret-token"));
}
