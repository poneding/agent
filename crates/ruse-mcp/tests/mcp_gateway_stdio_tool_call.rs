use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ruse_mcp::{
    McpClientStdioTransport, McpError, McpGateway, McpServerConfig, McpServerId, McpServerStatus,
    McpStdioClient, McpTransportConfig,
};
use ruse_tool::{ToolInput, ToolName, ToolRegistry};
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

fn config() -> McpServerConfig {
    McpServerConfig::new(
        server_id(),
        "Stdio Tools",
        McpTransportConfig::stdio("mcp-tools"),
    )
    .expect("config")
}

fn initialize_frame(id: i64) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
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

fn tools_list_frame(id: i64) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "read_file",
                    "title": "Read file",
                    "description": "Read a file through stdio MCP",
                    "inputSchema": {
                        "type": "object"
                    }
                }
            ]
        }
    })
    .to_string()
}

fn tool_call_frame(id: i64) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "output_json": {
                "content": "abcdefghijklmnopqrstuvwxyz"
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
async fn mcp_gateway_stdio_tool_call_discovers_registers_and_executes_remote_tool() {
    let config = config();
    let (transport, sent_frames) = RecordingTransport::from_inbound_frames([
        initialize_frame(1),
        tools_list_frame(2),
        tools_list_frame(3),
        tool_call_frame(4),
    ]);
    let client = Arc::new(McpStdioClient::new(transport));
    let gateway = McpGateway::new(Arc::clone(&client)).with_max_response_bytes(16);

    let report = gateway.discover_tools(std::slice::from_ref(&config)).await;

    assert_eq!(report.servers.len(), 1);
    assert_eq!(report.servers[0].status, McpServerStatus::Healthy);
    assert_eq!(report.tools.len(), 1);
    assert_eq!(report.tools[0].local_name, "mcp.stdio-tools.read_file");

    let mut registry = ToolRegistry::default();
    gateway
        .register_tools(&mut registry, &report)
        .expect("register");
    let tool = registry
        .get(&ToolName::new("mcp.stdio-tools.read_file").expect("tool name"))
        .expect("registered tool");
    let output = tool
        .execute(ToolInput {
            value: json!({
                "path": "README.md",
                "token": "sk-secret",
                "note": "prefix sk-secret suffix",
            }),
        })
        .expect("execute");

    assert_eq!(output.value["truncated"], true);
    assert!(
        output.value["content"]
            .as_str()
            .expect("bounded content")
            .len()
            <= 16
    );

    let call = sent_json(&sent_frames, 3);
    assert_eq!(call["id"], 4);
    assert_eq!(call["method"], "tools/call");
    assert_eq!(call["params"]["name"], "read_file");
    assert_eq!(call["params"]["arguments"]["path"], "README.md");
    assert_eq!(call["params"]["arguments"]["token"], "[redacted]");
    assert_eq!(
        call["params"]["arguments"]["note"],
        "prefix [redacted] suffix"
    );
}
