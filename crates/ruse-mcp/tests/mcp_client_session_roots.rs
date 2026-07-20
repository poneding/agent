use ruse_mcp::{
    FixtureMcpClient, McpClientRootProvider as _, McpInitializeResult, McpRoot, McpRootListResult,
    McpServerCapabilities, McpServerConfig, McpServerId, McpTransportConfig, list_mcp_client_roots,
};

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

#[tokio::test]
async fn mcp_client_session_roots_fixture_lists_configured_roots() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let roots =
        McpRootListResult::from_roots([
            McpRoot::new("file:///workspace/project", Some("Project")).expect("root")
        ]);
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_roots(server_id.clone(), roots.clone())
        .with_roots_list_changed(true);

    let listed = client.list_roots(&server_id).await.expect("roots");
    let capabilities = client.client_capabilities();

    assert_eq!(listed, roots);
    assert!(capabilities.roots.expect("roots capability").list_changed);
}

#[tokio::test]
async fn mcp_client_session_roots_fixture_returns_empty_roots_for_known_server() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let client = FixtureMcpClient::new().with_server(server_id.clone(), init(server_id.clone()));

    let listed = client.list_roots(&server_id).await.expect("roots");

    assert!(listed.roots.is_empty());
    assert!(client.client_capabilities().roots.is_none());
}

#[tokio::test]
async fn mcp_client_session_roots_fixture_rejects_unknown_server() {
    let client = FixtureMcpClient::new();
    let server_id = McpServerId::new("missing").expect("server id");

    let error = client
        .list_roots(&server_id)
        .await
        .expect_err("unknown server");

    assert!(error.to_string().contains("MCP server not found"));
}

#[tokio::test]
async fn mcp_client_session_roots_disabled_config_exposes_empty_roots() {
    let server_id = McpServerId::new("filesystem").expect("server id");
    let roots =
        McpRootListResult::from_roots([
            McpRoot::new("file:///workspace/project", Some("Project")).expect("root")
        ]);
    let disabled_config = config(server_id.clone()).disabled();
    let client = FixtureMcpClient::new()
        .with_server(server_id.clone(), init(server_id.clone()))
        .with_roots(server_id, roots);

    let listed = list_mcp_client_roots(&client, &disabled_config)
        .await
        .expect("disabled roots");

    assert!(listed.roots.is_empty());
}
