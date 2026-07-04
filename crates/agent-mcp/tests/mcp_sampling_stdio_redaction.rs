use agent_mcp::{
    FixtureMcpClient, MCP_SAMPLING_CREATE_MESSAGE_METHOD, McpCreateMessageResult, McpError,
    McpInitializeResult, McpJsonRpcFailure, McpSamplingContent, McpSamplingRole,
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

fn valid_request_frame(id: &str) -> String {
    request_frame(
        id,
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
        }),
    )
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

async fn failure_for(client: &FixtureMcpClient, config: &McpServerConfig, frame: &str) -> String {
    let response = handle_mcp_sampling_stdio_frame(client, config, frame).await;
    serde_json::from_str::<McpJsonRpcFailure>(response.trim_end()).expect("failure frame");
    response
}

#[tokio::test]
async fn mcp_sampling_stdio_redaction_hides_prompt_text_in_invalid_request_errors() {
    let server_id = server_id();
    let config = config(server_id.clone());
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, result());
    let secret_prompt = "sk-secret-token prompt text must stay private";
    let frame = request_frame(
        "invalid",
        json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": secret_prompt
                    }
                }
            ],
            "systemPrompt": secret_prompt,
            "maxTokens": 0
        }),
    );

    let response = failure_for(&client, &config, &frame).await;

    assert!(response.contains("invalid_max_tokens"));
    assert!(!response.contains(secret_prompt));
    assert!(!response.contains("sk-secret-token"));
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}

#[tokio::test]
async fn mcp_sampling_stdio_redaction_hides_provider_error_secrets() {
    let server_id = server_id();
    let config = config(server_id.clone());
    let raw_provider_error = "raw provider response body: invoice 12345 internal stack trace";
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_error(server_id, McpError::Client(raw_provider_error.to_string()));

    let response = failure_for(&client, &config, &valid_request_frame("provider")).await;
    let failure =
        serde_json::from_str::<McpJsonRpcFailure>(response.trim_end()).expect("failure frame");

    assert_eq!(failure.error.code, -32603);
    assert_eq!(failure.error.message, "sampling provider failed");
    assert!(!response.contains(raw_provider_error));
    assert!(!response.contains("invoice 12345"));
    assert_eq!(client.sampling_call_count("filesystem"), 1);
}

#[tokio::test]
async fn mcp_sampling_stdio_redaction_bounds_error_messages() {
    let server_id = server_id();
    let config = config(server_id.clone());
    let raw_provider_error = "raw provider response body: ".to_string() + &"x".repeat(2_048);
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_error(server_id, McpError::Client(raw_provider_error.clone()));

    let response = failure_for(&client, &config, &valid_request_frame("provider-long")).await;
    let failure =
        serde_json::from_str::<McpJsonRpcFailure>(response.trim_end()).expect("failure frame");

    assert!(failure.error.message.chars().count() <= 259);
    assert!(!response.contains(&raw_provider_error));
    assert!(!response.contains(&"x".repeat(512)));
}

#[tokio::test]
async fn mcp_sampling_stdio_redaction_rejects_secret_like_method_names_safely() {
    let server_id = server_id();
    let config = config(server_id.clone());
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, result());
    let frame = json!({
        "jsonrpc": "2.0",
        "id": "secret-method",
        "method": "sampling/sk-secret-method",
        "params": {}
    })
    .to_string();

    let response = failure_for(&client, &config, &frame).await;
    let failure =
        serde_json::from_str::<McpJsonRpcFailure>(response.trim_end()).expect("failure frame");

    assert_eq!(failure.error.code, -32601);
    assert_eq!(failure.error.message, "[redacted]");
    assert!(!response.contains("sk-secret-method"));
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}
