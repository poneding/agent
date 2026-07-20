use ruse_mcp::{
    FixtureMcpClient, McpClientRootProvider as _, McpCreateMessageRequestParams,
    McpCreateMessageResult, McpError, McpInitializeResult, McpRoot, McpRootListResult,
    McpSamplingContent, McpSamplingProvider as _, McpSamplingRole, McpServerCapabilities,
    McpServerConfig, McpServerId, McpTransportConfig, create_mcp_sampling_message,
};
use serde_json::json;

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

fn request(text: &str) -> McpCreateMessageRequestParams {
    McpCreateMessageRequestParams::new(
        vec![ruse_mcp::McpSamplingMessage::user(
            McpSamplingContent::text(text),
        )],
        128,
    )
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
async fn mcp_client_session_sampling_fixture_returns_configured_result() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let expected = result("fixture answer");
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id.clone(), expected.clone());

    let actual = client
        .create_message(&server_id, request("hello from server"))
        .await
        .expect("sampling result");

    assert_eq!(actual, expected);
    assert_eq!(client.sampling_call_count("filesystem"), 1);
}

#[tokio::test]
async fn mcp_client_session_sampling_fixture_rejects_unknown_server() {
    let client = FixtureMcpClient::new();
    let server_id = McpServerId::new("missing").expect("server id");

    let error = client
        .create_message(&server_id, request("hello from server"))
        .await
        .expect_err("unknown server");

    assert!(matches!(error, McpError::ServerNotFound(_)));
    assert_eq!(client.sampling_call_count("missing"), 0);
}

#[tokio::test]
async fn mcp_client_session_sampling_fixture_returns_configured_error() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_error(
            server_id.clone(),
            McpError::Client("fixture sampling failure".to_string()),
        );

    let error = client
        .create_message(&server_id, request("hello from server"))
        .await
        .expect_err("configured error");

    assert!(matches!(error, McpError::Client(_)));
    assert_eq!(client.sampling_call_count("filesystem"), 1);
}

#[tokio::test]
async fn mcp_client_session_sampling_disabled_config_rejects_before_provider() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let disabled_config = config(server_id.clone()).disabled();
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, result("unused"));

    let error = create_mcp_sampling_message(&client, &disabled_config, request("do not send"))
        .await
        .expect_err("disabled config");

    assert!(matches!(error, McpError::InvalidConfig(_)));
    assert!(error.to_string().contains("disabled"));
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}

#[tokio::test]
async fn mcp_client_session_sampling_validates_request_before_provider() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let enabled_config = config(server_id.clone());
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id, result("unused"));
    let mut invalid_request = request("secret prompt text should not leak");
    invalid_request.tools.push(json!({
        "name": "get_weather",
        "inputSchema": {
            "type": "object"
        }
    }));

    let error = create_mcp_sampling_message(&client, &enabled_config, invalid_request)
        .await
        .expect_err("invalid request");

    let rendered = format!("{error:?} {error}");
    assert!(matches!(error, McpError::InvalidServerRequest(_)));
    assert!(rendered.contains("unsupported_sampling_tools"));
    assert!(!rendered.contains("secret prompt text"));
    assert_eq!(client.sampling_call_count("filesystem"), 0);
}

#[tokio::test]
async fn mcp_client_session_sampling_tracks_calls_without_prompt_text() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let prompt = "sk-secret-token from server prompt";
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_sampling_result(server_id.clone(), result("fixture answer"));

    let _ = client
        .create_message(&server_id, request(prompt))
        .await
        .expect("sampling result");

    let rendered = format!("{client:?}");
    assert_eq!(client.sampling_call_count("filesystem"), 1);
    assert!(!rendered.contains(prompt));
    assert!(!rendered.contains("sk-secret-token"));
}

#[tokio::test]
async fn mcp_client_session_sampling_capabilities_combine_roots_and_sampling() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let roots =
        McpRootListResult::from_roots([
            McpRoot::new("file:///workspace/project", Some("Project")).expect("root")
        ]);
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_roots(server_id.clone(), roots.clone())
        .with_roots_list_changed(true)
        .with_sampling_result(server_id.clone(), result("fixture answer"));

    let listed = client.list_roots(&server_id).await.expect("roots");
    let capabilities = client.client_capabilities();

    assert_eq!(listed, roots);
    assert!(capabilities.roots.expect("roots").list_changed);
    let sampling = capabilities.sampling.expect("sampling");
    assert!(!sampling.tools);
    assert!(!sampling.context);
}
