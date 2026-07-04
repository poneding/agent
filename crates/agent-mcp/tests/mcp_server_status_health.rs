use agent_mcp::{
    MCP_SERVER_STATUS_HEALTH_SCHEMA_VERSION, McpServerStatusCapabilities, McpServerStatusHealth,
    McpServerStatusHealthDiagnosticSeverity, McpServerStatusSummary,
    mcp_server_status_health_report,
};

fn summary() -> McpServerStatusSummary {
    McpServerStatusSummary {
        available: true,
        protocol_version: "2025-11-25".to_string(),
        server_name: "agent".to_string(),
        server_version: "0.1.0".to_string(),
        initialized: true,
        capabilities: McpServerStatusCapabilities {
            tools: true,
            prompts: false,
            resources: true,
        },
        stdio_fixture_available: true,
        exported_tool_count: 1,
        exported_resource_count: 1,
        exported_tool_names: vec!["test.echo".to_string()],
        exported_resource_uris: vec!["agent://readme".to_string()],
    }
}

#[test]
fn mcp_server_status_health_reports_healthy_status() {
    let report = mcp_server_status_health_report(&summary());

    assert_eq!(
        report.schema_version,
        MCP_SERVER_STATUS_HEALTH_SCHEMA_VERSION
    );
    assert_eq!(report.status, McpServerStatusHealth::Healthy);
    assert!(report.diagnostics.is_empty());
    assert!(report.next_actions.is_empty());
}

#[test]
fn mcp_server_status_health_reports_not_initialized_and_no_exports() {
    let mut summary = summary();
    summary.initialized = false;
    summary.exported_tool_count = 0;
    summary.exported_resource_count = 0;
    summary.exported_tool_names.clear();
    summary.exported_resource_uris.clear();

    let report = mcp_server_status_health_report(&summary);

    assert_eq!(report.status, McpServerStatusHealth::Degraded);
    assert_eq!(
        report.next_actions,
        vec![
            "mcp.server.initialize".to_string(),
            "mcp.server.configure_exports".to_string()
        ]
    );
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == McpServerStatusHealthDiagnosticSeverity::Info
            && diagnostic.code == "mcp.server.not_initialized"
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == McpServerStatusHealthDiagnosticSeverity::Warning
            && diagnostic.code == "mcp.server.no_exports"
    }));
}

#[test]
fn mcp_server_status_health_reports_capability_export_mismatch_and_redacts() {
    let mut summary = summary();
    summary.capabilities.tools = false;
    summary.capabilities.resources = false;
    summary.exported_tool_names = vec!["token sk-secret tool".to_string()];

    let report = mcp_server_status_health_report(&summary);
    let serialized = serde_json::to_string(&report).expect("report json");

    assert_eq!(report.status, McpServerStatusHealth::Unhealthy);
    assert_eq!(report.next_actions, vec!["mcp.server.fix_capabilities"]);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == McpServerStatusHealthDiagnosticSeverity::Error
            && diagnostic.code == "mcp.server.tool_exports_without_capability"
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == McpServerStatusHealthDiagnosticSeverity::Error
            && diagnostic.code == "mcp.server.resource_exports_without_capability"
    }));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("token"));
}
