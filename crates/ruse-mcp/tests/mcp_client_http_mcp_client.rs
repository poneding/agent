use std::collections::BTreeMap;

use ruse_mcp::{
    FixtureMcpClientHttpTransport, McpClient as _, McpClientHttpResponse, McpHttpClient,
    McpServerConfig, McpServerId, McpServerStatus, McpTransportConfig, check_mcp_server_health,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("http-client").expect("server id")
}

fn other_server_id() -> McpServerId {
    McpServerId::new("other-http-client").expect("server id")
}

fn config() -> McpServerConfig {
    let mut config = McpServerConfig::new(
        server_id(),
        "HTTP Client",
        McpTransportConfig::StreamableHttp {
            url: "http://localhost/mcp".to_string(),
            headers: BTreeMap::new(),
        },
    )
    .expect("config");
    config.timeout_ms = 1_000;
    config
}

fn initialize_result() -> serde_json::Value {
    json!({
        "protocolVersion": "2025-11-25",
        "serverInfo": {
            "name": "http-server",
            "version": "1.2.3"
        },
        "capabilities": {
            "tools": {},
            "prompts": false,
            "resources": {}
        }
    })
}

fn initialize_response(id: i64) -> McpClientHttpResponse {
    McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": initialize_result()
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
                        "description": "Read through HTTP MCP",
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

#[tokio::test]
async fn mcp_client_http_mcp_client_maps_initialize_and_tools_list() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(initialize_response(1)),
        Ok(tools_response(2)),
    ]);
    let client = McpHttpClient::new(transport);

    let init = client.initialize(&config).await.expect("initialize");
    let tools = client.list_tools(&config.id).await.expect("tools");

    assert_eq!(init.server_id, config.id);
    assert_eq!(init.protocol_version, "2025-11-25");
    assert_eq!(init.server_name, "http-server");
    assert_eq!(init.server_version.as_deref(), Some("1.2.3"));
    assert!(init.capabilities.tools);
    assert!(!init.capabilities.prompts);
    assert!(init.capabilities.resources);
    assert_eq!(tools.server_id, config.id);
    assert_eq!(tools.tools.len(), 1);
    assert_eq!(tools.tools[0].name, "read_file");
    assert_eq!(tools.tools[0].title.as_deref(), Some("Read file"));
}

#[tokio::test]
async fn mcp_client_http_mcp_client_health_reports_healthy() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([
        Ok(initialize_response(1)),
        Ok(tools_response(2)),
    ]);
    let client = McpHttpClient::new(transport);

    let health = check_mcp_server_health(&client, &config).await;

    assert_eq!(health.status, McpServerStatus::Healthy);
    assert!(health.message.contains("http-server"));
    assert!(health.message.contains("1 tools"));
}

#[tokio::test]
async fn mcp_client_http_mcp_client_rejects_list_before_initialize() {
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::default());

    let error = client
        .list_tools(&server_id())
        .await
        .expect_err("uninitialized");

    assert!(error.to_string().contains("not initialized"));
}

#[tokio::test]
async fn mcp_client_http_mcp_client_rejects_wrong_server() {
    let config = config();
    let transport = FixtureMcpClientHttpTransport::from_responses([Ok(initialize_response(1))]);
    let client = McpHttpClient::new(transport);
    client.initialize(&config).await.expect("initialize");

    let error = client
        .list_tools(&other_server_id())
        .await
        .expect_err("wrong server");

    assert!(error.to_string().contains("not initialized"));
}

#[tokio::test]
async fn mcp_client_http_mcp_client_invalid_initialize_shape_is_redacted() {
    let config = config();
    let secret_protocol = "sk-secret-token-should-not-leak";
    let response = McpClientHttpResponse::new(
        200,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": secret_protocol,
                "serverInfo": {
                    "version": "1.2.3"
                },
                "capabilities": {
                    "tools": true
                }
            }
        }),
    )
    .expect("response");
    let client = McpHttpClient::new(FixtureMcpClientHttpTransport::from_responses([Ok(
        response,
    )]));

    let error = client
        .initialize(&config)
        .await
        .expect_err("invalid initialize");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP initialize result"));
    assert!(!message.contains(secret_protocol));
    assert!(!message.contains("sk-secret-token"));
}
