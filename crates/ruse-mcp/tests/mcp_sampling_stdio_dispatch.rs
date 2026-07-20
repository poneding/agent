use ruse_mcp::{
    FixtureMcpClient, MCP_SAMPLING_CREATE_MESSAGE_METHOD, McpCreateMessageResult,
    McpInitializeResult, McpJsonRpcError, McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcSuccess,
    McpSamplingContent, McpSamplingRole, McpServerCapabilities, McpServerConfig, McpServerId,
    McpTransportConfig, handle_mcp_sampling_stdio_frame,
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

fn request_frame(id: serde_json::Value) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": MCP_SAMPLING_CREATE_MESSAGE_METHOD,
        "params": {
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "summarize this"
                    }
                }
            ],
            "maxTokens": 128
        }
    })
    .to_string()
}

fn notification_frame() -> String {
    json!({
        "jsonrpc": "2.0",
        "method": MCP_SAMPLING_CREATE_MESSAGE_METHOD,
        "params": {
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "notify only"
                    }
                }
            ],
            "maxTokens": 64
        }
    })
    .to_string()
}

fn result(text: &str) -> McpCreateMessageResult {
    McpCreateMessageResult::new(
        McpSamplingRole::Assistant,
        McpSamplingContent::text(text),
        "fixture-model",
        Some("endTurn"),
    )
    .expect("result")
}

#[tokio::test]
async fn mcp_sampling_stdio_dispatch_returns_success_frame() {
    let server_id = server_id();
    let expected = result("fixture answer");
    let config = config(server_id.clone());
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, expected.clone());

    let frame =
        handle_mcp_sampling_stdio_frame(&client, &config, &request_frame(json!("sample-1"))).await;

    assert!(frame.ends_with('\n'));
    let response =
        serde_json::from_str::<McpJsonRpcSuccess>(frame.trim_end()).expect("success frame");
    assert_eq!(response.id, McpJsonRpcId::String("sample-1".to_string()));
    assert_eq!(
        response.result,
        serde_json::to_value(expected).expect("result json")
    );
    assert_eq!(client.sampling_call_count("filesystem"), 1);
}

#[tokio::test]
async fn mcp_sampling_stdio_dispatch_preserves_numeric_ids() {
    let server_id = server_id();
    let expected = result("numeric id answer");
    let config = config(server_id.clone());
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, expected);

    let frame = handle_mcp_sampling_stdio_frame(&client, &config, &request_frame(json!(42))).await;

    let response =
        serde_json::from_str::<McpJsonRpcSuccess>(frame.trim_end()).expect("success frame");
    assert_eq!(response.id, McpJsonRpcId::Number(42));
    assert_eq!(client.sampling_call_count("filesystem"), 1);
}

#[tokio::test]
async fn mcp_sampling_stdio_dispatch_returns_empty_for_notification() {
    let server_id = server_id();
    let config = config(server_id.clone());
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, result("unused"));

    let frame = handle_mcp_sampling_stdio_frame(&client, &config, &notification_frame()).await;

    assert_eq!(frame, "");
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}

#[tokio::test]
async fn mcp_sampling_stdio_dispatch_rejects_disabled_config_before_provider() {
    let server_id = server_id();
    let config = config(server_id.clone()).disabled();
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, result("unused"));

    let frame =
        handle_mcp_sampling_stdio_frame(&client, &config, &request_frame(json!("disabled"))).await;

    let response =
        serde_json::from_str::<McpJsonRpcFailure>(frame.trim_end()).expect("failure frame");
    assert_eq!(response.id, McpJsonRpcId::String("disabled".to_string()));
    assert_eq!(
        response.error,
        McpJsonRpcError::internal_error("MCP server is disabled")
    );
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}
