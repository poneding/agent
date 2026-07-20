use ruse_mcp::{
    FixtureMcpClient, MCP_SAMPLING_CREATE_MESSAGE_METHOD, McpCreateMessageResult,
    McpInitializeResult, McpJsonRpcFailure, McpJsonRpcId, McpSamplingContent, McpSamplingRole,
    McpServerCapabilities, McpServerConfig, McpServerId, McpTransportConfig,
    handle_mcp_sampling_stdio_frame,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("filesystem").expect("server id")
}

fn init(server_id: McpServerId) -> McpInitializeResult {
    McpInitializeResult::new(
        server_id,
        "2025-11-25",
        "filesystem",
        Some("1.0.0".to_string()),
        McpServerCapabilities {
            tools: true,
            prompts: false,
            resources: false,
        },
    )
}

fn config(server_id: McpServerId) -> McpServerConfig {
    McpServerConfig::new(
        server_id,
        "Filesystem",
        McpTransportConfig::stdio("mcp-filesystem"),
    )
    .expect("config")
}

fn result() -> McpCreateMessageResult {
    McpCreateMessageResult::new(
        McpSamplingRole::Assistant,
        McpSamplingContent::text("unused"),
        "fixture-model",
        Some("endTurn"),
    )
    .expect("result")
}

fn fixture() -> (FixtureMcpClient, McpServerConfig) {
    let server_id = server_id();
    let config = config(server_id.clone());
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, result());
    (client, config)
}

fn request_frame(id: &str, params: serde_json::Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": MCP_SAMPLING_CREATE_MESSAGE_METHOD,
        "params": params
    })
    .to_string()
}

fn basic_params() -> serde_json::Value {
    json!({
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": "safe prompt"
                }
            }
        ],
        "maxTokens": 128
    })
}

async fn failure_for(frame: &str) -> (McpJsonRpcFailure, FixtureMcpClient) {
    let (client, config) = fixture();

    let response = handle_mcp_sampling_stdio_frame(&client, &config, frame).await;
    let failure =
        serde_json::from_str::<McpJsonRpcFailure>(response.trim_end()).expect("failure frame");

    (failure, client)
}

#[tokio::test]
async fn mcp_sampling_stdio_validation_rejects_tools_before_provider() {
    let mut params = basic_params();
    params["tools"] = json!([
        {
            "name": "get_weather",
            "inputSchema": {
                "type": "object"
            }
        }
    ]);
    let frame = request_frame("tools", params);

    let (failure, client) = failure_for(&frame).await;

    assert_eq!(failure.id, McpJsonRpcId::String("tools".to_string()));
    assert_eq!(failure.error.code, -32600);
    assert!(failure.error.message.contains("unsupported_sampling_tools"));
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}

#[tokio::test]
async fn mcp_sampling_stdio_validation_rejects_context_before_provider() {
    let mut params = basic_params();
    params["includeContext"] = json!("thisServer");
    let frame = request_frame("context", params);

    let (failure, client) = failure_for(&frame).await;

    assert_eq!(failure.id, McpJsonRpcId::String("context".to_string()));
    assert_eq!(failure.error.code, -32600);
    assert!(failure.error.message.contains("unsupported_context"));
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}

#[tokio::test]
async fn mcp_sampling_stdio_validation_rejects_invalid_tokens_before_provider() {
    let mut params = basic_params();
    params["maxTokens"] = json!(0);
    let frame = request_frame("tokens", params);

    let (failure, client) = failure_for(&frame).await;

    assert_eq!(failure.id, McpJsonRpcId::String("tokens".to_string()));
    assert_eq!(failure.error.code, -32600);
    assert!(failure.error.message.contains("invalid_max_tokens"));
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}

#[tokio::test]
async fn mcp_sampling_stdio_validation_rejects_malformed_params_before_provider() {
    let frame = request_frame(
        "malformed",
        json!({
            "messages": "not an array",
            "maxTokens": 128
        }),
    );

    let (failure, client) = failure_for(&frame).await;

    assert_eq!(failure.error.code, -32602);
    assert_eq!(
        failure.error.message,
        "invalid sampling/createMessage params"
    );
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}
