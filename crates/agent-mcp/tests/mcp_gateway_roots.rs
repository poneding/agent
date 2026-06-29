use std::sync::Arc;

use agent_mcp::{
    FixtureMcpClient, McpError, McpGateway, McpInitializeResult, McpRoot, McpRootListResult,
    McpServerCapabilities, McpServerConfig, McpServerId, McpServerStatus, McpToolDescriptor,
    McpTransportConfig,
};
use agent_tool::ToolRegistry;

fn server_config(id: &str) -> McpServerConfig {
    McpServerConfig::new(
        McpServerId::new(id).unwrap(),
        format!("{id} server"),
        McpTransportConfig::stdio(format!("mcp-{id}")),
    )
    .unwrap()
}

fn descriptor(server_id: McpServerId, name: &str) -> McpToolDescriptor {
    McpToolDescriptor::new(
        server_id,
        name,
        Some(name),
        format!("{name} via MCP"),
        serde_json::json!({"type": "object"}),
    )
    .unwrap()
}

fn init(server_id: McpServerId) -> McpInitializeResult {
    McpInitializeResult::new(
        server_id,
        "2025-11-25",
        "fixture",
        Some("1.0.0".to_string()),
        McpServerCapabilities {
            tools: true,
            prompts: false,
            resources: false,
        },
    )
}

#[tokio::test]
async fn mcp_gateway_roots_reports_roots_for_healthy_servers() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let roots =
        McpRootListResult::from_roots([
            McpRoot::new("file:///workspace/project", Some("Project")).expect("root")
        ]);
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(
                server_id.clone(),
                vec![descriptor(server_id.clone(), "read_file")],
            )
            .with_roots(server_id.clone(), roots),
    );

    let gateway = McpGateway::new(client);
    let report = gateway.discover_tools(&[server_config("filesystem")]).await;
    let value = serde_json::to_value(&report).expect("report json");

    assert_eq!(report.roots.len(), 1);
    assert_eq!(report.roots[0].server_id.as_str(), "filesystem");
    assert_eq!(report.roots[0].uri, "file:///workspace/project");
    assert_eq!(report.roots[0].name.as_deref(), Some("Project"));
    assert_eq!(value["roots"][0]["uri"], "file:///workspace/project");
    assert_eq!(report.tools.len(), 1);
    assert!(report.diagnostics.is_empty());
}

#[tokio::test]
async fn mcp_gateway_roots_excludes_disabled_and_unhealthy_servers() {
    let healthy_id = McpServerId::new("filesystem").unwrap();
    let roots =
        McpRootListResult::from_roots([
            McpRoot::new("file:///workspace/project", Some("Project")).expect("root")
        ]);
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(healthy_id.clone(), init(healthy_id.clone()))
            .with_tools(
                healthy_id.clone(),
                vec![descriptor(healthy_id.clone(), "read")],
            )
            .with_roots(healthy_id, roots),
    );
    let disabled = server_config("filesystem").disabled();
    let unhealthy = server_config("missing");

    let gateway = McpGateway::new(client);
    let report = gateway.discover_tools(&[disabled, unhealthy]).await;

    assert!(report.roots.is_empty());
    assert!(report.tools.is_empty());
    assert_eq!(report.servers[0].status, McpServerStatus::Disabled);
    assert_eq!(report.servers[1].status, McpServerStatus::Unhealthy);
}

#[tokio::test]
async fn mcp_gateway_roots_provider_errors_are_redacted_and_do_not_block_tools() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(
                server_id.clone(),
                vec![descriptor(server_id.clone(), "read_file")],
            )
            .with_root_error(
                server_id,
                McpError::Client("api_key sk-secret root discovery failed".to_string()),
            ),
    );

    let gateway = McpGateway::new(client);
    let report = gateway.discover_tools(&[server_config("filesystem")]).await;
    let mut registry = ToolRegistry::default();
    gateway
        .register_tools(&mut registry, &report)
        .expect("register tools");

    assert!(report.roots.is_empty());
    assert_eq!(report.tools.len(), 1);
    assert_eq!(registry.definitions().len(), 1);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].code, "root_discovery_failed");
    assert!(report.diagnostics[0].message.contains("[redacted]"));
    assert!(!report.diagnostics[0].message.contains("sk-secret"));
    assert!(!report.diagnostics[0].message.contains("api_key"));
}
