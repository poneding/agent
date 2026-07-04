use agent_mcp::{
    McpClientHttpOAuthRefreshAction, McpClientHttpOAuthTokenDiagnosticCode,
    McpClientHttpOAuthTokenDiagnosticSeverity, McpClientHttpOAuthTokenLease,
    build_mcp_client_http_oauth_refresh_plan,
};
use serde_json::json;

fn lease(expires_at_unix_ms: Option<u64>, scopes: &[&str]) -> McpClientHttpOAuthTokenLease {
    McpClientHttpOAuthTokenLease::new(
        "refresh-secret-token",
        expires_at_unix_ms,
        scopes.iter().map(|scope| (*scope).to_string()).collect(),
    )
    .expect("token lease")
}

#[test]
fn mcp_client_http_oauth_token_refresh_reports_usable_token() {
    let plan = build_mcp_client_http_oauth_refresh_plan(
        &lease(Some(2_000), &["mcp:tools"]),
        1_000,
        100,
        &["mcp:tools".to_string()],
    )
    .expect("refresh plan");

    assert_eq!(plan.action, McpClientHttpOAuthRefreshAction::UseToken);
    assert_eq!(
        serde_json::to_value(&plan).expect("json"),
        json!({
            "action": "use_token",
            "diagnostics": [
                {
                    "severity": "info",
                    "code": "token_usable",
                    "message": "MCP HTTP OAuth token lease is usable"
                }
            ]
        })
    );
}

#[test]
fn mcp_client_http_oauth_token_refresh_reports_expired_token_without_leaking() {
    let plan = build_mcp_client_http_oauth_refresh_plan(
        &lease(Some(900), &["mcp:tools"]),
        1_000,
        100,
        &["mcp:tools".to_string()],
    )
    .expect("refresh plan");

    assert_eq!(plan.action, McpClientHttpOAuthRefreshAction::Expired);
    assert_eq!(
        plan.diagnostics[0].severity,
        McpClientHttpOAuthTokenDiagnosticSeverity::Error
    );
    assert_eq!(
        plan.diagnostics[0].code,
        McpClientHttpOAuthTokenDiagnosticCode::TokenExpired
    );

    let message = serde_json::to_string(&plan).expect("json");
    assert!(!message.contains("refresh-secret-token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_refresh_reports_refresh_soon() {
    let plan = build_mcp_client_http_oauth_refresh_plan(
        &lease(Some(1_050), &["mcp:tools"]),
        1_000,
        100,
        &["mcp:tools".to_string()],
    )
    .expect("refresh plan");

    assert_eq!(plan.action, McpClientHttpOAuthRefreshAction::RefreshSoon);
    assert_eq!(
        plan.diagnostics[0].severity,
        McpClientHttpOAuthTokenDiagnosticSeverity::Warning
    );
    assert_eq!(
        plan.diagnostics[0].code,
        McpClientHttpOAuthTokenDiagnosticCode::TokenRefreshRecommended
    );
}

#[test]
fn mcp_client_http_oauth_token_refresh_reports_unknown_expiry() {
    let plan = build_mcp_client_http_oauth_refresh_plan(
        &lease(None, &["mcp:tools"]),
        1_000,
        100,
        &["mcp:tools".to_string()],
    )
    .expect("refresh plan");

    assert_eq!(plan.action, McpClientHttpOAuthRefreshAction::UseToken);
    assert_eq!(
        plan.diagnostics[0].code,
        McpClientHttpOAuthTokenDiagnosticCode::TokenExpiryUnknown
    );
}

#[test]
fn mcp_client_http_oauth_token_refresh_reports_missing_required_scope() {
    let plan = build_mcp_client_http_oauth_refresh_plan(
        &lease(Some(2_000), &["mcp:tools"]),
        1_000,
        100,
        &["mcp:resources".to_string()],
    )
    .expect("refresh plan");

    assert_eq!(plan.action, McpClientHttpOAuthRefreshAction::ScopeMismatch);
    assert_eq!(
        plan.diagnostics[0].severity,
        McpClientHttpOAuthTokenDiagnosticSeverity::Error
    );
    assert_eq!(
        plan.diagnostics[0].code,
        McpClientHttpOAuthTokenDiagnosticCode::RequiredScopeMissing
    );
}

#[test]
fn mcp_client_http_oauth_token_refresh_rejects_invalid_required_scopes_without_leaking() {
    let error = build_mcp_client_http_oauth_refresh_plan(
        &lease(Some(2_000), &["mcp:tools"]),
        1_000,
        100,
        &["scope secret-token".to_string()],
    )
    .expect_err("invalid required scope");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token lease"));
    assert!(!message.contains("scope secret-token"));
    assert!(!message.contains("secret-token"));
}
