use agent_mcp::{
    McpClientHttpOAuthTokenDiagnosticCode, McpClientHttpOAuthTokenDiagnosticSeverity,
    McpClientHttpOAuthTokenLease, build_mcp_client_http_oauth_token_status,
};
use serde_json::json;

#[test]
fn mcp_client_http_oauth_token_accepts_valid_lease() {
    let lease = McpClientHttpOAuthTokenLease::new(
        "access-secret-token",
        Some(1_800_000),
        vec!["mcp:tools".to_string(), "mcp:resources".to_string()],
    )
    .expect("token lease");

    assert_eq!(lease.access_token, "access-secret-token");
    assert_eq!(lease.expires_at_unix_ms, Some(1_800_000));
    assert_eq!(lease.scopes, ["mcp:tools", "mcp:resources"]);
}

#[test]
fn mcp_client_http_oauth_token_rejects_invalid_tokens_without_leaking() {
    for token in ["", "   ", "access-secret-token\nnext"] {
        let error =
            McpClientHttpOAuthTokenLease::new(token, None, Vec::new()).expect_err("invalid token");
        let message = error.to_string();

        assert!(message.contains("invalid MCP client HTTP OAuth token lease"));
        assert!(!message.contains("access-secret-token"));
    }
}

#[test]
fn mcp_client_http_oauth_token_rejects_invalid_scopes_without_leaking_token() {
    let error = McpClientHttpOAuthTokenLease::new(
        "scope-secret-token",
        Some(1_800_000),
        vec!["mcp:tools".to_string(), "bad scope".to_string()],
    )
    .expect_err("invalid scope");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token lease"));
    assert!(!message.contains("scope-secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_debug_is_redacted() {
    let lease = McpClientHttpOAuthTokenLease::new(
        "debug-secret-token",
        Some(1_800_000),
        vec!["mcp:tools".to_string()],
    )
    .expect("token lease");
    let debug = format!("{lease:?}");

    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("debug-secret-token"));
    assert!(!debug.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_status_serializes_stable_usable_shape() {
    let lease = McpClientHttpOAuthTokenLease::new(
        "status-secret-token",
        Some(1_800_000),
        vec!["mcp:tools".to_string()],
    )
    .expect("token lease");

    let status = build_mcp_client_http_oauth_token_status(&lease);

    assert!(status.usable);
    assert_eq!(
        status.diagnostics[0].severity,
        McpClientHttpOAuthTokenDiagnosticSeverity::Info
    );
    assert_eq!(
        status.diagnostics[0].code,
        McpClientHttpOAuthTokenDiagnosticCode::TokenUsable
    );
    assert_eq!(
        serde_json::to_value(&status).expect("json"),
        json!({
            "usable": true,
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
fn mcp_client_http_oauth_token_status_reports_unknown_expiry_without_token() {
    let lease = McpClientHttpOAuthTokenLease::new(
        "unknown-expiry-secret-token",
        None,
        vec!["mcp:tools".to_string()],
    )
    .expect("token lease");

    let status = build_mcp_client_http_oauth_token_status(&lease);

    assert!(status.usable);
    assert_eq!(
        status.diagnostics[0].severity,
        McpClientHttpOAuthTokenDiagnosticSeverity::Warning
    );
    assert_eq!(
        status.diagnostics[0].code,
        McpClientHttpOAuthTokenDiagnosticCode::TokenExpiryUnknown
    );

    let message = serde_json::to_string(&status).expect("json");
    assert!(!message.contains("unknown-expiry-secret-token"));
    assert!(!message.contains("secret-token"));
}
