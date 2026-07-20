use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use ruse_mcp::{
    McpClient as _, McpClientHttpRequest, McpClientHttpResponse, McpClientHttpResponseEnvelope,
    McpClientHttpTransport, McpError, McpGateway, McpGatewayClient as _, McpHttpClient,
    McpServerConfig, McpServerId, McpServerStatus, McpToolCallRequest, McpTransportConfig,
};
use ruse_tool::{ToolInput, ToolName, ToolRegistry};
use async_trait::async_trait;
use serde_json::{Value, json};

struct RecordingTransport {
    inbound_responses: VecDeque<Result<McpClientHttpResponseEnvelope, McpError>>,
    sent_requests: Arc<Mutex<Vec<McpClientHttpRequest>>>,
}

impl RecordingTransport {
    fn from_responses<I, R>(responses: I) -> (Self, Arc<Mutex<Vec<McpClientHttpRequest>>>)
    where
        I: IntoIterator<Item = Result<R, McpError>>,
        R: Into<McpClientHttpResponseEnvelope>,
    {
        let sent_requests = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                inbound_responses: responses
                    .into_iter()
                    .map(|response| response.map(Into::into))
                    .collect(),
                sent_requests: Arc::clone(&sent_requests),
            },
            sent_requests,
        )
    }
}

#[async_trait]
impl McpClientHttpTransport for RecordingTransport {
    async fn send_request(
        &mut self,
        request: McpClientHttpRequest,
    ) -> Result<McpClientHttpResponseEnvelope, McpError> {
        self.sent_requests
            .lock()
            .expect("sent requests")
            .push(request);
        self.inbound_responses
            .pop_front()
            .unwrap_or_else(|| Err(McpError::Client("missing response".to_string())))
    }
}

fn server_id() -> McpServerId {
    McpServerId::new("http-tools").expect("server id")
}

fn config() -> McpServerConfig {
    McpServerConfig::new(
        server_id(),
        "HTTP Tools",
        McpTransportConfig::StreamableHttp {
            url: "http://localhost/mcp".to_string(),
            headers: BTreeMap::new(),
        },
    )
    .expect("config")
}

fn initialize_response(id: i64) -> McpClientHttpResponse {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-11-25",
                "serverInfo": {
                    "name": "http-tool-server",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": {}
                }
            }
        }),
    )
    .expect("response")
}

fn tools_response(id: i64) -> McpClientHttpResponse {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {
                        "name": "read_file",
                        "title": "Read file",
                        "description": "Read a file through HTTP MCP",
                        "inputSchema": {
                            "type": "object"
                        }
                    }
                ]
            }
        }),
    )
    .expect("response")
}

fn tool_call_response(id: i64) -> McpClientHttpResponse {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "output_json": {
                    "content": "abcdefghijklmnopqrstuvwxyz"
                }
            }
        }),
    )
    .expect("response")
}

fn raw_secret_result_response(id: i64, raw_result: &str) -> McpClientHttpResponse {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": raw_result
        }),
    )
    .expect("response")
}

fn sent_body(sent_requests: &Arc<Mutex<Vec<McpClientHttpRequest>>>, index: usize) -> Value {
    sent_requests.lock().expect("sent requests")[index]
        .body_json
        .clone()
}

#[tokio::test]
async fn mcp_gateway_http_tool_call_discovers_registers_and_executes_remote_tool() {
    let config = config();
    let (transport, sent_requests) = RecordingTransport::from_responses([
        Ok(initialize_response(1)),
        Ok(tools_response(2)),
        Ok(tools_response(3)),
        Ok(tool_call_response(4)),
    ]);
    let client = Arc::new(McpHttpClient::new(transport));
    let gateway = McpGateway::new(Arc::clone(&client)).with_max_response_bytes(16);

    let report = gateway.discover_tools(std::slice::from_ref(&config)).await;

    assert_eq!(report.servers.len(), 1);
    assert_eq!(report.servers[0].status, McpServerStatus::Healthy);
    assert_eq!(report.tools.len(), 1);
    assert_eq!(report.tools[0].local_name, "mcp.http-tools.read_file");
    assert!(report.diagnostics.is_empty());

    let mut registry = ToolRegistry::default();
    gateway
        .register_tools(&mut registry, &report)
        .expect("register");
    let tool = registry
        .get(&ToolName::new("mcp.http-tools.read_file").expect("tool name"))
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

    let call = sent_body(&sent_requests, 3);
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

#[tokio::test]
async fn mcp_gateway_http_tool_call_invalid_result_shape_is_redacted() {
    let config = config();
    let raw_result = "sk-secret-token should not leak from malformed result";
    let (transport, _sent_requests) = RecordingTransport::from_responses([
        Ok(initialize_response(1)),
        Ok(raw_secret_result_response(2, raw_result)),
    ]);
    let client = McpHttpClient::new(transport);
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

    assert!(message.contains("invalid MCP client HTTP tools/call result"));
    assert!(!message.contains(raw_result));
    assert!(!message.contains("sk-secret-token"));
}
