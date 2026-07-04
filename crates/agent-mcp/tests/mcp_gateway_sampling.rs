use std::sync::Arc;

use agent_mcp::{
    FixtureMcpClient, McpClientSamplingCapability, McpCreateMessageResult, McpError, McpGateway,
    McpInitializeResult, McpSamplingContent, McpSamplingRole, McpServerCapabilities,
    McpServerConfig, McpServerId, McpServerStatus, McpToolDescriptor, McpTransportConfig,
};

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

fn sampling_result(text: &str) -> McpCreateMessageResult {
    McpCreateMessageResult::new(
        McpSamplingRole::Assistant,
        McpSamplingContent::text(text),
        "fixture-model",
        Some("endTurn"),
    )
    .expect("sampling result")
}

#[tokio::test]
async fn mcp_gateway_sampling_reports_availability_for_healthy_servers() {
    let alpha = McpServerId::new("alpha").unwrap();
    let zeta = McpServerId::new("zeta").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(alpha.clone(), init(alpha.clone()))
            .with_tools(alpha.clone(), vec![descriptor(alpha.clone(), "read_alpha")])
            .with_server(zeta.clone(), init(zeta.clone()))
            .with_tools(zeta.clone(), vec![descriptor(zeta.clone(), "read_zeta")])
            .with_sampling_result(alpha, sampling_result("unused")),
    );

    let gateway = McpGateway::new(client);
    let report = gateway
        .discover_tools(&[server_config("zeta"), server_config("alpha")])
        .await;

    assert_eq!(report.sampling.len(), 2);
    assert_eq!(report.sampling[0].server_id.as_str(), "alpha");
    assert!(report.sampling[0].available);
    assert!(!report.sampling[0].tools_supported);
    assert!(!report.sampling[0].context_supported);
    assert_eq!(report.sampling[1].server_id.as_str(), "zeta");
    assert!(report.diagnostics.is_empty());
}

#[tokio::test]
async fn mcp_gateway_sampling_excludes_disabled_and_unhealthy_servers() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(
                server_id.clone(),
                vec![descriptor(server_id.clone(), "read_file")],
            )
            .with_sampling_result(server_id, sampling_result("unused")),
    );

    let gateway = McpGateway::new(client);
    let report = gateway
        .discover_tools(&[
            server_config("filesystem").disabled(),
            server_config("missing"),
        ])
        .await;

    assert!(report.sampling.is_empty());
    assert!(report.tools.is_empty());
    assert_eq!(report.servers[0].status, McpServerStatus::Disabled);
    assert_eq!(report.servers[1].status, McpServerStatus::Unhealthy);
}

#[tokio::test]
async fn mcp_gateway_sampling_errors_are_redacted_and_do_not_block_tools() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(
                server_id.clone(),
                vec![descriptor(server_id.clone(), "read_file")],
            )
            .with_sampling_error(
                server_id.clone(),
                McpError::Client("sk-secret-token sampling provider failure".to_string()),
            ),
    );

    let gateway = McpGateway::new(Arc::clone(&client));
    let report = gateway.discover_tools(&[server_config("filesystem")]).await;
    let rendered = serde_json::to_string(&report).expect("report json");

    assert_eq!(report.tools.len(), 1);
    assert_eq!(report.sampling.len(), 1);
    assert_eq!(client.sampling_call_count("filesystem"), 0);
    assert!(!rendered.contains("sk-secret-token"));
    assert!(!rendered.contains("provider failure"));
}

#[tokio::test]
async fn mcp_gateway_sampling_policy_extension_diagnostics_do_not_block_tools() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(
                server_id.clone(),
                vec![descriptor(server_id.clone(), "read_file")],
            )
            .with_sampling_capability(McpClientSamplingCapability::with_tools_and_context()),
    );

    let gateway = McpGateway::new(client);
    let report = gateway.discover_tools(&[server_config("filesystem")]).await;

    assert_eq!(report.tools.len(), 1);
    assert_eq!(report.sampling.len(), 1);
    assert!(report.sampling[0].tools_supported);
    assert!(report.sampling[0].context_supported);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "sampling_tools_rejected_by_policy")
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "sampling_context_rejected_by_policy")
    );
}

#[tokio::test]
async fn mcp_gateway_sampling_report_serializes_stable_shape() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(
                server_id.clone(),
                vec![descriptor(server_id.clone(), "read_file")],
            )
            .with_sampling_result(server_id, sampling_result("unused")),
    );

    let gateway = McpGateway::new(client);
    let report = gateway.discover_tools(&[server_config("filesystem")]).await;
    let value = serde_json::to_value(&report).expect("report json");

    assert_eq!(value["sampling"][0]["server_id"], "filesystem");
    assert_eq!(value["sampling"][0]["available"], true);
    assert_eq!(value["sampling"][0]["tools_supported"], false);
    assert_eq!(value["sampling"][0]["context_supported"], false);
    assert!(value["sampling"][0]["message"].as_str().unwrap().len() <= 256);
}
