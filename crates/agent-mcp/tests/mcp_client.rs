use agent_mcp::McpClient as _;

#[tokio::test]
async fn mcp_client_validates_server_config() {
    assert!(agent_mcp::McpServerId::new("filesystem").is_ok());
    assert!(agent_mcp::McpServerId::new("FileSystem").is_err());
}

#[tokio::test]
async fn mcp_client_serializes_transport_shape() {
    use std::collections::BTreeMap;

    let config = agent_mcp::McpServerConfig::new(
        agent_mcp::McpServerId::new("docs").unwrap(),
        "Docs",
        agent_mcp::McpTransportConfig::Stdio {
            command: "mcp-docs".to_string(),
            args: vec!["--stdio".to_string()],
            env: BTreeMap::from([(
                "API_TOKEN".to_string(),
                agent_mcp::McpEnvValue::SecretRef("docs.token".to_string()),
            )]),
        },
    )
    .unwrap();

    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["transport"]["kind"], "stdio");
    assert_eq!(json["transport"]["env"]["API_TOKEN"]["kind"], "secret_ref");
}

#[tokio::test]
async fn mcp_client_deserialize_rejects_invalid_config() {
    let empty_display = serde_json::json!({
        "id": "docs",
        "display_name": " ",
        "transport": {
            "kind": "stdio",
            "command": "mcp-docs",
            "args": [],
            "env": {}
        },
        "enabled": true,
        "timeout_ms": 30_000
    });
    assert!(serde_json::from_value::<agent_mcp::McpServerConfig>(empty_display).is_err());

    let invalid_url = serde_json::json!({
        "id": "docs",
        "display_name": "Docs",
        "transport": {
            "kind": "streamable_http",
            "url": "file:///tmp/socket",
            "headers": {}
        },
        "enabled": true,
        "timeout_ms": 30_000
    });
    assert!(serde_json::from_value::<agent_mcp::McpServerConfig>(invalid_url).is_err());

    let zero_timeout = serde_json::json!({
        "id": "docs",
        "display_name": "Docs",
        "transport": {
            "kind": "stdio",
            "command": "mcp-docs",
            "args": [],
            "env": {}
        },
        "enabled": true,
        "timeout_ms": 0
    });
    assert!(serde_json::from_value::<agent_mcp::McpServerConfig>(zero_timeout).is_err());
}

#[tokio::test]
async fn mcp_client_fixture_initializes_and_lists_tools() {
    let server_id = agent_mcp::McpServerId::new("filesystem").unwrap();
    let config = agent_mcp::McpServerConfig::new(
        server_id.clone(),
        "Filesystem",
        agent_mcp::McpTransportConfig::stdio("mcp-filesystem"),
    )
    .unwrap();

    let client = agent_mcp::FixtureMcpClient::new()
        .with_server(
            server_id.clone(),
            agent_mcp::McpInitializeResult::new(
                server_id.clone(),
                "2025-11-25",
                "filesystem",
                Some("1.0.0".to_string()),
                agent_mcp::McpServerCapabilities {
                    tools: true,
                    prompts: false,
                    resources: false,
                },
            ),
        )
        .with_tools(
            server_id.clone(),
            vec![
                agent_mcp::McpToolDescriptor::new(
                    server_id.clone(),
                    "read_file",
                    Some("Read file"),
                    "Read a file from the MCP server",
                    serde_json::json!({"type": "object"}),
                )
                .unwrap(),
            ],
        );

    let init = client.initialize(&config).await.unwrap();
    let tools = client.list_tools(&server_id).await.unwrap();

    assert_eq!(init.protocol_version, "2025-11-25");
    assert_eq!(tools.tools.len(), 1);
    assert_eq!(tools.tools[0].name, "read_file");
}

#[tokio::test]
async fn mcp_client_health_reports_disabled_healthy_and_unhealthy() {
    let server_id = agent_mcp::McpServerId::new("filesystem").unwrap();
    let enabled = agent_mcp::McpServerConfig::new(
        server_id.clone(),
        "Filesystem",
        agent_mcp::McpTransportConfig::stdio("mcp-filesystem"),
    )
    .unwrap();
    let disabled = enabled.clone().disabled();
    let empty_client = agent_mcp::FixtureMcpClient::new();

    let disabled_health = agent_mcp::check_mcp_server_health(&empty_client, &disabled).await;
    assert_eq!(disabled_health.status, agent_mcp::McpServerStatus::Disabled);

    let unhealthy = agent_mcp::check_mcp_server_health(&empty_client, &enabled).await;
    assert_eq!(unhealthy.status, agent_mcp::McpServerStatus::Unhealthy);

    let healthy_client = agent_mcp::FixtureMcpClient::new()
        .with_server(
            server_id.clone(),
            agent_mcp::McpInitializeResult::new(
                server_id.clone(),
                "2025-11-25",
                "filesystem",
                None::<String>,
                agent_mcp::McpServerCapabilities {
                    tools: true,
                    prompts: false,
                    resources: false,
                },
            ),
        )
        .with_tools(server_id, Vec::new());

    let healthy = agent_mcp::check_mcp_server_health(&healthy_client, &enabled).await;
    assert_eq!(healthy.status, agent_mcp::McpServerStatus::Healthy);
}
