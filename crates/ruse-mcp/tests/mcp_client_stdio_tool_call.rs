use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use ruse_mcp::{
    McpClient as _, McpClientStdioTransport, McpError, McpGatewayClient as _, McpServerConfig,
    McpServerId, McpStdioClient, McpToolCallRequest, McpTransportConfig,
};
use async_trait::async_trait;
use serde_json::{Value, json};

struct RecordingTransport {
    inbound_frames: VecDeque<String>,
    sent_frames: Arc<Mutex<Vec<String>>>,
}

impl RecordingTransport {
    fn from_inbound_frames<I, S>(frames: I) -> (Self, Arc<Mutex<Vec<String>>>)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let sent_frames = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                inbound_frames: frames.into_iter().map(Into::into).collect(),
                sent_frames: Arc::clone(&sent_frames),
            },
            sent_frames,
        )
    }
}

#[async_trait]
impl McpClientStdioTransport for RecordingTransport {
    async fn send_frame(&mut self, frame: String) -> Result<(), McpError> {
        self.sent_frames.lock().expect("sent frames").push(frame);
        Ok(())
    }

    async fn recv_frame(&mut self) -> Result<Option<String>, McpError> {
        Ok(self.inbound_frames.pop_front())
    }
}

fn server_id() -> McpServerId {
    McpServerId::new("stdio-tools").expect("server id")
}

fn other_server_id() -> McpServerId {
    McpServerId::new("other-tools").expect("server id")
}

fn config() -> McpServerConfig {
    McpServerConfig::new(
        server_id(),
        "Stdio Tools",
        McpTransportConfig::stdio("mcp-tools"),
    )
    .expect("config")
}

fn initialize_frame() -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "protocolVersion": "2025-11-25",
            "serverInfo": {
                "name": "tool-server",
                "version": "0.1.0"
            },
            "capabilities": {
                "tools": {}
            }
        }
    })
    .to_string()
}

fn sent_json(sent_frames: &Arc<Mutex<Vec<String>>>, index: usize) -> Value {
    let sent = sent_frames.lock().expect("sent frames");
    serde_json::from_str(sent[index].trim_end()).expect("sent frame json")
}

#[tokio::test]
async fn mcp_client_stdio_tool_call_sends_params_and_maps_output_json_result() {
    let config = config();
    let (transport, sent_frames) = RecordingTransport::from_inbound_frames([
        initialize_frame(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "output_json": {
                    "content": "hello from remote"
                }
            }
        })
        .to_string(),
    ]);
    let client = McpStdioClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    let output = client
        .call_tool(McpToolCallRequest {
            server_id: config.id.clone(),
            remote_tool_name: "read_file".to_string(),
            input_json: json!({
                "path": "README.md"
            }),
        })
        .await
        .expect("tool call");

    assert_eq!(output.output_json["content"], "hello from remote");

    let request = sent_json(&sent_frames, 1);
    assert_eq!(request["id"], 2);
    assert_eq!(request["method"], "tools/call");
    assert_eq!(request["params"]["name"], "read_file");
    assert_eq!(request["params"]["arguments"]["path"], "README.md");
}

#[tokio::test]
async fn mcp_client_stdio_tool_call_wraps_standard_object_result() {
    let config = config();
    let (transport, _sent_frames) = RecordingTransport::from_inbound_frames([
        initialize_frame(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": "standard content"
                    }
                ],
                "isError": false
            }
        })
        .to_string(),
    ]);
    let client = McpStdioClient::new(transport);

    client.initialize(&config).await.expect("initialize");
    let output = client
        .call_tool(McpToolCallRequest {
            server_id: config.id.clone(),
            remote_tool_name: "read_file".to_string(),
            input_json: json!({}),
        })
        .await
        .expect("tool call");

    assert_eq!(output.output_json["content"][0]["text"], "standard content");
    assert_eq!(output.output_json["isError"], false);
}

#[tokio::test]
async fn mcp_client_stdio_tool_call_rejects_uninitialized_session_before_write() {
    let (transport, sent_frames) = RecordingTransport::from_inbound_frames(Vec::<String>::new());
    let client = McpStdioClient::new(transport);

    let error = client
        .call_tool(McpToolCallRequest {
            server_id: server_id(),
            remote_tool_name: "read_file".to_string(),
            input_json: json!({}),
        })
        .await
        .expect_err("uninitialized");

    assert!(error.to_string().contains("not initialized"));
    assert!(sent_frames.lock().expect("sent frames").is_empty());
}

#[tokio::test]
async fn mcp_client_stdio_tool_call_rejects_wrong_server_before_write() {
    let config = config();
    let (transport, sent_frames) = RecordingTransport::from_inbound_frames([initialize_frame()]);
    let client = McpStdioClient::new(transport);
    client.initialize(&config).await.expect("initialize");

    let error = client
        .call_tool(McpToolCallRequest {
            server_id: other_server_id(),
            remote_tool_name: "read_file".to_string(),
            input_json: json!({}),
        })
        .await
        .expect_err("wrong server");

    assert!(error.to_string().contains("not initialized"));
    assert_eq!(sent_frames.lock().expect("sent frames").len(), 1);
}

#[tokio::test]
async fn mcp_client_stdio_tool_call_failure_error_is_redacted() {
    let config = config();
    let remote_secret = "sk-secret-token from remote failure";
    let (transport, _sent_frames) = RecordingTransport::from_inbound_frames([
        initialize_frame(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "error": {
                "code": -32000,
                "message": remote_secret
            }
        })
        .to_string(),
    ]);
    let client = McpStdioClient::new(transport);
    client.initialize(&config).await.expect("initialize");

    let error = client
        .call_tool(McpToolCallRequest {
            server_id: config.id.clone(),
            remote_tool_name: "read_file".to_string(),
            input_json: json!({}),
        })
        .await
        .expect_err("remote failure");
    let message = error.to_string();

    assert!(message.contains("tools/call failed with code -32000"));
    assert!(!message.contains(remote_secret));
    assert!(!message.contains("sk-secret-token"));
}

#[tokio::test]
async fn mcp_client_stdio_tool_call_invalid_result_shape_is_redacted() {
    let config = config();
    let raw_result = "sk-secret-token should not leak from malformed result";
    let (transport, _sent_frames) = RecordingTransport::from_inbound_frames([
        initialize_frame(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "result": raw_result
        })
        .to_string(),
    ]);
    let client = McpStdioClient::new(transport);
    client.initialize(&config).await.expect("initialize");

    let error = client
        .call_tool(McpToolCallRequest {
            server_id: config.id.clone(),
            remote_tool_name: "read_file".to_string(),
            input_json: json!({}),
        })
        .await
        .expect_err("invalid result");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client stdio tools/call result"));
    assert!(!message.contains(raw_result));
    assert!(!message.contains("sk-secret-token"));
}
