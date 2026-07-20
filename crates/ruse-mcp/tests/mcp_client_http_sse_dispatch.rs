use std::collections::BTreeMap;

use ruse_mcp::{
    FixtureMcpClientHttpTransport, McpClient as _, McpClientHttpSseResponse, McpGatewayClient as _,
    McpHttpClient, McpServerConfig, McpServerId, McpToolCallRequest, McpTransportConfig,
    parse_mcp_client_http_sse_events,
};
use serde_json::{Value, json};

fn server_id() -> McpServerId {
    McpServerId::new("http-sse-client").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP SSE Client",
        McpTransportConfig::StreamableHttp {
            url: "http://localhost/mcp".to_string(),
            headers: BTreeMap::new(),
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

fn sse_response(id: i64, result: Value) -> McpClientHttpSseResponse {
    let frame = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    });
    let body = format!("event: message\ndata: {}\n\n", frame);
    McpClientHttpSseResponse::new(parse_mcp_client_http_sse_events(&body).expect("events"))
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2025-11-25",
        "serverInfo": {
            "name": "http-sse-server",
            "version": "1.2.3"
        },
        "capabilities": {
            "tools": {}
        }
    })
}

fn tools_result() -> Value {
    json!({
        "tools": [
            {
                "name": "read_file",
                "title": "Read file",
                "description": "Read through SSE HTTP MCP",
                "inputSchema": {
                    "type": "object"
                }
            }
        ]
    })
}

#[tokio::test]
async fn mcp_client_http_sse_dispatch_maps_initialize_and_tools_list() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(sse_response(1, initialize_result())),
        Ok(sse_response(2, tools_result())),
    ]);
    let client = McpHttpClient::new(transport);

    let init = client.initialize(&config).await.expect("initialize");
    let tools = client.list_tools(&config.id).await.expect("tools");

    assert_eq!(init.server_id, config.id);
    assert_eq!(init.server_name, "http-sse-server");
    assert_eq!(init.server_version.as_deref(), Some("1.2.3"));
    assert!(init.capabilities.tools);
    assert_eq!(tools.tools.len(), 1);
    assert_eq!(tools.tools[0].name, "read_file");
}

#[tokio::test]
async fn mcp_client_http_sse_dispatch_maps_tool_call() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(sse_response(1, initialize_result())),
        Ok(sse_response(
            2,
            json!({
                "output_json": {
                    "content": "from sse"
                }
            }),
        )),
    ]);
    let client = McpHttpClient::new(transport);

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
        .expect("call");

    assert_eq!(output.output_json["content"], "from sse");
}

#[tokio::test]
async fn mcp_client_http_sse_dispatch_rejects_missing_matching_id_safely() {
    let config = config();
    let secret_result = "sk-secret-token should not leak";
    let transport = FixtureMcpClientHttpTransport::from_responses([Ok(sse_response(
        99,
        json!({
            "value": secret_result
        }),
    ))]);
    let client = McpHttpClient::new(transport);

    let error = client
        .initialize(&config)
        .await
        .expect_err("missing matching id");
    let message = error.to_string();

    assert!(message.contains("matching response id missing"));
    assert!(!message.contains("99"));
    assert!(!message.contains(secret_result));
    assert!(!message.contains("sk-secret-token"));
}
