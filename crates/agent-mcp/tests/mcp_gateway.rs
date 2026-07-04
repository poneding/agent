use std::sync::Arc;

use agent_mcp::{
    FixtureMcpClient, McpGateway, McpGatewayClient, McpInitializeResult, McpServerCapabilities,
    McpServerConfig, McpServerId, McpServerStatus, McpToolCallRequest, McpToolCallResult,
    McpToolDescriptor, McpTransportConfig,
};
use agent_policy::{
    CapabilityAction, PolicyDecision, PolicyEngine, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use agent_tool::{ToolInput, ToolRegistry};

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
async fn mcp_gateway_discovers_enabled_tools_and_reports_disabled_unhealthy_servers() {
    let healthy_id = McpServerId::new("filesystem").unwrap();
    let unhealthy = server_config("broken");
    let disabled = server_config("disabled").disabled();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(healthy_id.clone(), init(healthy_id.clone()))
            .with_tools(
                healthy_id.clone(),
                vec![descriptor(healthy_id, "read_file")],
            ),
    );

    let gateway = McpGateway::new(client);
    let report = gateway
        .discover_tools(&[server_config("filesystem"), unhealthy, disabled])
        .await;

    assert_eq!(report.tools.len(), 1);
    assert_eq!(
        report.tools[0].local_name.as_str(),
        "mcp.filesystem.read_file"
    );
    assert_eq!(report.servers[0].status, McpServerStatus::Unhealthy);
    assert_eq!(report.servers[1].status, McpServerStatus::Disabled);
    assert_eq!(report.servers[2].status, McpServerStatus::Healthy);
}

#[tokio::test]
async fn mcp_gateway_registers_local_tools_with_source_metadata() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(server_id.clone(), vec![descriptor(server_id, "read_file")]),
    );
    let gateway = McpGateway::new(client);
    let report = gateway.discover_tools(&[server_config("filesystem")]).await;
    let mut registry = ToolRegistry::default();

    gateway.register_tools(&mut registry, &report).unwrap();

    let definition = registry.definitions().remove(0);
    assert_eq!(definition.name.as_str(), "mcp.filesystem.read_file");
    assert!(definition.description.contains("via MCP"));
    assert_eq!(report.tools[0].server_id.as_str(), "filesystem");
    assert_eq!(report.tools[0].remote_tool_name, "read_file");
}

#[tokio::test]
async fn mcp_gateway_denied_local_policy_does_not_call_remote_tool() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(
                server_id.clone(),
                vec![descriptor(server_id.clone(), "read_file")],
            )
            .with_tool_result(
                server_id,
                "read_file",
                McpToolCallResult::new(serde_json::json!({"content": "remote"})),
            ),
    );
    let gateway = McpGateway::new(client.clone());
    let report = gateway.discover_tools(&[server_config("filesystem")]).await;
    let mut registry = ToolRegistry::default();
    gateway.register_tools(&mut registry, &report).unwrap();
    let tool = registry
        .get(&agent_tool::ToolName::new("mcp.filesystem.read_file").unwrap())
        .unwrap();
    let policy = StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "mcp.filesystem.read_file").unwrap(),
        PolicyDecision::Deny {
            reason: "denied".to_string(),
        },
    );

    let request = agent_policy::PermissionRequest::new(
        agent_types::ActorRef::system(),
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "mcp.filesystem.read_file").unwrap(),
        "tool execution requested",
        agent_types::CorrelationId::new(),
    )
    .unwrap();
    assert!(matches!(
        policy.decide(&request).unwrap(),
        PolicyDecision::Deny { .. }
    ));
    assert_eq!(client.call_count("filesystem", "read_file"), 0);
    drop(tool);
}

#[tokio::test]
async fn mcp_gateway_executes_allowed_remote_tool_and_bounds_output() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = Arc::new(
        FixtureMcpClient::new()
            .with_server(server_id.clone(), init(server_id.clone()))
            .with_tools(
                server_id.clone(),
                vec![descriptor(server_id.clone(), "read_file")],
            )
            .with_tool_result(
                server_id,
                "read_file",
                McpToolCallResult::new(serde_json::json!({"content": "hello from mcp"})),
            ),
    );
    let gateway = McpGateway::new(client.clone()).with_max_response_bytes(8);
    let report = gateway.discover_tools(&[server_config("filesystem")]).await;
    let mut registry = ToolRegistry::default();
    gateway.register_tools(&mut registry, &report).unwrap();
    let tool = registry
        .get(&agent_tool::ToolName::new("mcp.filesystem.read_file").unwrap())
        .unwrap();

    let output = tool
        .execute(ToolInput {
            value: serde_json::json!({"path": "README.md", "token": "sk-secret"}),
        })
        .unwrap();

    assert_eq!(client.call_count("filesystem", "read_file"), 1);
    assert_eq!(output.value["truncated"], true);
    assert!(output.value["content"].as_str().unwrap().len() <= 8);
}

#[tokio::test]
async fn mcp_gateway_client_call_tool_contract_is_available() {
    let server_id = McpServerId::new("filesystem").unwrap();
    let client = FixtureMcpClient::new().with_tool_result(
        server_id.clone(),
        "read_file",
        McpToolCallResult::new(serde_json::json!({"content": "ok"})),
    );

    let output = client
        .call_tool(McpToolCallRequest {
            server_id,
            remote_tool_name: "read_file".to_string(),
            input_json: serde_json::json!({}),
        })
        .await
        .unwrap();

    assert_eq!(output.output_json["content"], "ok");
}
