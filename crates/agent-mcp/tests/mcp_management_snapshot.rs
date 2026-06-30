use std::collections::BTreeMap;

use agent_mcp::{
    McpEnvValue, McpGatewayDiagnostic, McpGatewayReport, McpGatewayRootDescriptor,
    McpGatewaySamplingDescriptor, McpManagementAuthStatus, McpManagementDiagnosticSnapshot,
    McpManagementExtensionHealth, McpManagementServerSnapshotInput, McpManagementSnapshot,
    McpManagementStreamStatus, McpManagementTransportKind, McpServerConfig, McpServerHealth,
    McpServerId, McpServerStatus, McpTransportConfig,
};

fn stdio_config() -> McpServerConfig {
    McpServerConfig::new(
        McpServerId::new("stdio-server").expect("server id"),
        "Stdio Server",
        McpTransportConfig::stdio("mcp-stdio"),
    )
    .expect("config")
}

fn http_config() -> McpServerConfig {
    let mut headers = BTreeMap::new();
    headers.insert(
        "Authorization".to_string(),
        McpEnvValue::Literal("Bearer sk-secret-token".to_string()),
    );
    McpServerConfig::new(
        McpServerId::new("http-server").expect("server id"),
        "HTTP Server",
        McpTransportConfig::StreamableHttp {
            url: "https://example.com/mcp".to_string(),
            headers,
        },
    )
    .expect("config")
}

fn report() -> McpGatewayReport {
    let stdio_id = McpServerId::new("stdio-server").expect("server id");
    let http_id = McpServerId::new("http-server").expect("server id");
    McpGatewayReport {
        servers: vec![
            McpServerHealth {
                server_id: stdio_id.clone(),
                status: McpServerStatus::Healthy,
                message: "healthy".to_string(),
            },
            McpServerHealth {
                server_id: http_id.clone(),
                status: McpServerStatus::Unhealthy,
                message: "remote token sk-secret-token failed".to_string(),
            },
        ],
        tools: Vec::new(),
        roots: vec![McpGatewayRootDescriptor {
            server_id: stdio_id.clone(),
            uri: "file:///workspace/project".to_string(),
            name: Some("Project".to_string()),
        }],
        sampling: vec![McpGatewaySamplingDescriptor {
            server_id: stdio_id.clone(),
            available: true,
            tools_supported: false,
            context_supported: false,
            message: "sampling available".to_string(),
        }],
        diagnostics: vec![McpGatewayDiagnostic {
            severity: "error".to_string(),
            code: "token_failure".to_string(),
            message: "authorization bearer sk-secret-token failed".to_string(),
            server_id: Some(http_id),
        }],
    }
}

#[test]
fn mcp_management_snapshot_summarizes_servers() {
    let configs = vec![stdio_config(), http_config()];
    let inputs = vec![(
        McpServerId::new("http-server").expect("server id"),
        McpManagementServerSnapshotInput::default()
            .with_extension_health(McpManagementExtensionHealth::NeedsAttention)
            .with_extra_diagnostic(McpManagementDiagnosticSnapshot::new(
                "warning",
                "extension_health",
                "extension requires review",
            )),
    )];

    let snapshot =
        McpManagementSnapshot::from_gateway_report_with_inputs(&configs, &report(), &inputs);

    assert_eq!(snapshot.servers.len(), 2);
    let http = snapshot
        .servers
        .iter()
        .find(|server| server.server_id.as_str() == "http-server")
        .expect("http server");
    assert_eq!(
        http.transport_kind,
        McpManagementTransportKind::StreamableHttp
    );
    assert_eq!(http.auth_status, McpManagementAuthStatus::Ready);
    assert_eq!(http.stream_status, McpManagementStreamStatus::Idle);
    assert_eq!(
        http.extension_health,
        McpManagementExtensionHealth::NeedsAttention
    );
    let stdio = snapshot
        .servers
        .iter()
        .find(|server| server.server_id.as_str() == "stdio-server")
        .expect("stdio server");
    assert_eq!(stdio.roots.len(), 1);
    assert!(stdio.sampling.available);
}

#[test]
fn mcp_management_snapshot_is_redacted_and_bounded() {
    let configs = vec![http_config()];
    let snapshot = McpManagementSnapshot::from_gateway_report(&configs, &report());

    let json = serde_json::to_string(&snapshot).expect("snapshot json");
    assert!(!json.contains("sk-secret-token"));
    assert!(!json.contains("Bearer"));
    assert!(!json.contains("authorization bearer"));
    assert!(json.contains("[redacted]"));
    assert!(snapshot.servers[0].last_diagnostics.len() <= 8);
}
