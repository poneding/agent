use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ruse_mcp::{
    McpClientStdioProcessTransport, McpClientStdioResponseFrame, McpClientStdioSession,
    McpEnvValue, McpServerConfig, McpServerId, McpTransportConfig,
};
use serde_json::json;
use tempfile::TempDir;

fn compile_helper(temp: &TempDir) -> PathBuf {
    let source = temp.path().join("mcp_stdio_helper.rs");
    fs::write(
        &source,
        r##"
use std::env;
use std::io::{self, BufRead, Write};
use std::process;

fn main() {
    match env::args().nth(1).as_deref() {
        Some("serve-once") => {
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            if lines.next().transpose().expect("stdin").is_none() {
                process::exit(65);
            }
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(br#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}
"#)
                .expect("stdout");
            stdout.flush().expect("flush");
        }
        Some("exit-after-read") => {
            let stdin = io::stdin();
            let mut line = String::new();
            let _ = stdin.lock().read_line(&mut line);
            eprintln!("sk-secret-token stderr should not leak");
            process::exit(7);
        }
        _ => process::exit(64),
    }
}
"##,
    )
    .expect("write helper source");

    let binary = temp
        .path()
        .join(format!("mcp-stdio-helper{}", std::env::consts::EXE_SUFFIX));
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
    McpServerId::new("process-fixture").expect("server id")
}

fn stdio_config(command: &Path, mode: &str) -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "Process Fixture",
        McpTransportConfig::Stdio {
            command: command.display().to_string(),
            args: vec![mode.to_string()],
            env: BTreeMap::new(),
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

#[tokio::test]
async fn mcp_client_stdio_process_exchanges_frame_with_fixture() {
    let temp = tempfile::tempdir().expect("tempdir");
    let helper = compile_helper(&temp);
    let config = stdio_config(&helper, "serve-once");
    let transport = McpClientStdioProcessTransport::spawn(&config)
        .await
        .expect("spawn transport");
    let mut session = McpClientStdioSession::new(transport);

    let response = session
        .send_request("initialize", json!({}))
        .await
        .expect("process response");

    match response {
        McpClientStdioResponseFrame::Success(success) => {
            assert_eq!(success.result["ok"], true);
        }
        other => panic!("expected success, got {other:?}"),
    }

    session.transport_mut().close().await.expect("close");
}

#[tokio::test]
async fn mcp_client_stdio_process_rejects_disabled_config_before_spawn() {
    let temp = tempfile::tempdir().expect("tempdir");
    let helper = compile_helper(&temp);
    let config = stdio_config(&helper, "serve-once").disabled();

    let error = match McpClientStdioProcessTransport::spawn(&config).await {
        Ok(_) => panic!("disabled config should be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("disabled"));
}

#[tokio::test]
async fn mcp_client_stdio_process_rejects_non_stdio_config_before_spawn() {
    let config = McpServerConfig::new(
        server_id(),
        "HTTP Fixture",
        McpTransportConfig::StreamableHttp {
            url: "http://localhost:3000/mcp".to_string(),
            headers: BTreeMap::new(),
        },
    )
    .expect("config");

    let error = match McpClientStdioProcessTransport::spawn(&config).await {
        Ok(_) => panic!("non-stdio config should be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("requires stdio config"));
}

#[tokio::test]
async fn mcp_client_stdio_process_exit_before_response_is_redacted() {
    let temp = tempfile::tempdir().expect("tempdir");
    let helper = compile_helper(&temp);
    let config = stdio_config(&helper, "exit-after-read");
    let transport = McpClientStdioProcessTransport::spawn(&config)
        .await
        .expect("spawn transport");
    let mut session = McpClientStdioSession::new(transport);

    let error = session
        .send_request("initialize", json!({}))
        .await
        .expect_err("process should exit before response");
    let message = error.to_string();

    assert!(message.contains("process exited before response"));
    assert!(message.contains("status 7"));
    assert!(!message.contains("sk-secret-token"));
    assert!(!message.contains("stderr should not leak"));
}

#[tokio::test]
async fn mcp_client_stdio_process_rejects_unresolved_secret_env() {
    let temp = tempfile::tempdir().expect("tempdir");
    let helper = compile_helper(&temp);
    let mut env = BTreeMap::new();
    env.insert(
        "API_TOKEN".to_string(),
        McpEnvValue::SecretRef("mcp.token".to_string()),
    );
    let config = McpServerConfig::new(
        server_id(),
        "Secret Fixture",
        McpTransportConfig::Stdio {
            command: helper.display().to_string(),
            args: vec!["serve-once".to_string()],
            env,
        },
    )
    .expect("config");

    let error = match McpClientStdioProcessTransport::spawn(&config).await {
        Ok(_) => panic!("secret refs should be resolved before spawn"),
        Err(error) => error,
    };

    let message = error.to_string();
    assert!(message.contains("unresolved secret ref"));
    assert!(!message.contains("mcp.token"));
    assert!(!message.contains("API_TOKEN"));
}
