use std::collections::BTreeMap;

use ruse_mcp::{
    McpClientHttpAuthorizationDiagnosticCode, McpClientHttpAuthorizationDiagnosticSeverity,
    McpClientHttpOAuthMetadata, McpClientHttpProtectedResourceMetadata, McpEnvValue,
    McpServerConfig, McpServerId, McpTransportConfig, build_mcp_client_http_authorization_report,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("mcp-auth").expect("server id")
}

fn streamable_config(headers: BTreeMap<String, McpEnvValue>) -> McpServerConfig {
    McpServerConfig::new(
        server_id(),
        "MCP Auth",
        McpTransportConfig::StreamableHttp {
            url: "https://mcp.example.com".to_string(),
            headers,
        },
    )
    .expect("config")
}

fn oauth_metadata() -> McpClientHttpOAuthMetadata {
    McpClientHttpOAuthMetadata {
        issuer: Some("https://auth.example.com".to_string()),
        authorization_endpoint: Some("https://auth.example.com/authorize".to_string()),
        token_endpoint: Some("https://auth.example.com/token".to_string()),
        registration_endpoint: None,
        scopes_supported: vec!["mcp:tools".to_string()],
        token_endpoint_auth_methods_supported: vec!["none".to_string()],
    }
}

fn protected_resource_metadata() -> McpClientHttpProtectedResourceMetadata {
    McpClientHttpProtectedResourceMetadata {
        resource: Some("https://mcp.example.com".to_string()),
        authorization_servers: vec!["https://auth.example.com".to_string()],
        scopes_supported: vec!["mcp:tools".to_string()],
    }
}

#[test]
fn mcp_client_http_authorization_diagnostics_serializes_stable_ready_report() {
    let report = build_mcp_client_http_authorization_report(
        &streamable_config(BTreeMap::new()),
        Some(&oauth_metadata()),
        Some(&protected_resource_metadata()),
    );

    assert!(report.authorized);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].severity,
        McpClientHttpAuthorizationDiagnosticSeverity::Info
    );
    assert_eq!(
        report.diagnostics[0].code,
        McpClientHttpAuthorizationDiagnosticCode::AuthorizationReady
    );
    assert_eq!(
        serde_json::to_value(&report).expect("json"),
        json!({
            "authorized": true,
            "diagnostics": [
                {
                    "severity": "info",
                    "code": "authorization_ready",
                    "message": "MCP HTTP authorization metadata is usable"
                }
            ]
        })
    );
}

#[test]
fn mcp_client_http_authorization_diagnostics_reports_missing_metadata() {
    let report =
        build_mcp_client_http_authorization_report(&streamable_config(BTreeMap::new()), None, None);

    assert!(!report.authorized);
    assert_eq!(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>(),
        [
            McpClientHttpAuthorizationDiagnosticCode::MissingProtectedResourceMetadata,
            McpClientHttpAuthorizationDiagnosticCode::MissingOAuthMetadata,
        ]
    );
    assert!(
        report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity
                == McpClientHttpAuthorizationDiagnosticSeverity::Warning)
    );
}

#[test]
fn mcp_client_http_authorization_diagnostics_reports_missing_token_endpoint_and_scopes() {
    let oauth = McpClientHttpOAuthMetadata {
        token_endpoint: None,
        scopes_supported: Vec::new(),
        ..oauth_metadata()
    };
    let protected_resource = McpClientHttpProtectedResourceMetadata {
        scopes_supported: Vec::new(),
        ..protected_resource_metadata()
    };

    let report = build_mcp_client_http_authorization_report(
        &streamable_config(BTreeMap::new()),
        Some(&oauth),
        Some(&protected_resource),
    );

    assert!(!report.authorized);
    assert_eq!(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>(),
        [
            McpClientHttpAuthorizationDiagnosticCode::MissingProtectedResourceScopes,
            McpClientHttpAuthorizationDiagnosticCode::MissingTokenEndpoint,
            McpClientHttpAuthorizationDiagnosticCode::MissingOAuthScopes,
        ]
    );
}

#[test]
fn mcp_client_http_authorization_diagnostics_reports_header_conflicts_without_leaking_values() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_string(),
        McpEnvValue::Literal("Bearer secret-token".to_string()),
    );
    let report = build_mcp_client_http_authorization_report(
        &streamable_config(headers),
        Some(&oauth_metadata()),
        Some(&protected_resource_metadata()),
    );

    assert!(!report.authorized);
    assert_eq!(
        report.diagnostics[0].severity,
        McpClientHttpAuthorizationDiagnosticSeverity::Error
    );
    assert_eq!(
        report.diagnostics[0].code,
        McpClientHttpAuthorizationDiagnosticCode::AuthorizationHeaderConflict
    );

    let message = serde_json::to_string(&report).expect("json");
    assert!(!message.contains("Bearer secret-token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_authorization_diagnostics_reports_secret_ref_header_conflicts_without_leaking() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "Authorization".to_string(),
        McpEnvValue::SecretRef("mcp.secret.token".to_string()),
    );
    let report = build_mcp_client_http_authorization_report(
        &streamable_config(headers),
        Some(&oauth_metadata()),
        Some(&protected_resource_metadata()),
    );

    assert!(!report.authorized);
    assert_eq!(
        report.diagnostics[0].code,
        McpClientHttpAuthorizationDiagnosticCode::AuthorizationHeaderConflict
    );

    let message = serde_json::to_string(&report).expect("json");
    assert!(!message.contains("mcp.secret.token"));
    assert!(!message.contains("secret"));
    assert!(!message.contains("token"));
}
