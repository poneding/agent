use agent_mcp::{
    McpClientHttpOAuthTokenResponse, build_mcp_client_http_oauth_token_lease_from_exchange_response,
};

#[test]
fn mcp_client_http_oauth_token_exchange_lease_converts_with_expiry() {
    let response = McpClientHttpOAuthTokenResponse {
        access_token: "access-secret-token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in_seconds: Some(3600),
        refresh_token: Some("refresh-secret-token".to_string()),
        scope: vec!["mcp:tools".to_string(), "mcp:resources".to_string()],
    };

    let lease =
        build_mcp_client_http_oauth_token_lease_from_exchange_response(&response, 1_000_000)
            .expect("token lease");

    assert_eq!(lease.access_token, "access-secret-token");
    assert_eq!(lease.expires_at_unix_ms, Some(4_600_000));
    assert_eq!(lease.scopes, ["mcp:tools", "mcp:resources"]);
}

#[test]
fn mcp_client_http_oauth_token_exchange_lease_converts_without_expiry() {
    let response = McpClientHttpOAuthTokenResponse {
        access_token: "access-secret-token".to_string(),
        token_type: "bearer".to_string(),
        expires_in_seconds: None,
        refresh_token: None,
        scope: Vec::new(),
    };

    let lease =
        build_mcp_client_http_oauth_token_lease_from_exchange_response(&response, 1_000_000)
            .expect("token lease");

    assert_eq!(lease.expires_at_unix_ms, None);
    assert!(lease.scopes.is_empty());
}

#[test]
fn mcp_client_http_oauth_token_exchange_lease_rejects_unsupported_token_type() {
    let response = McpClientHttpOAuthTokenResponse {
        access_token: "access-secret-token".to_string(),
        token_type: "mac".to_string(),
        expires_in_seconds: Some(60),
        refresh_token: None,
        scope: Vec::new(),
    };

    let error =
        build_mcp_client_http_oauth_token_lease_from_exchange_response(&response, 1_000_000)
            .expect_err("unsupported token type");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token lease"));
    assert!(!message.contains("access-secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_exchange_lease_omits_refresh_token() {
    let response = McpClientHttpOAuthTokenResponse {
        access_token: "access-secret-token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in_seconds: Some(60),
        refresh_token: Some("refresh-secret-token".to_string()),
        scope: Vec::new(),
    };

    let lease =
        build_mcp_client_http_oauth_token_lease_from_exchange_response(&response, 1_000_000)
            .expect("token lease");
    let debug = format!("{lease:?}");

    assert!(!debug.contains("refresh-secret-token"));
    assert!(!debug.contains("access-secret-token"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn mcp_client_http_oauth_token_exchange_lease_saturates_expiry() {
    let response = McpClientHttpOAuthTokenResponse {
        access_token: "access-secret-token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in_seconds: Some(u64::MAX),
        refresh_token: None,
        scope: Vec::new(),
    };

    let lease = build_mcp_client_http_oauth_token_lease_from_exchange_response(&response, u64::MAX)
        .expect("token lease");

    assert_eq!(lease.expires_at_unix_ms, Some(u64::MAX));
}

#[test]
fn mcp_client_http_oauth_token_exchange_lease_redacts_secret_like_scopes() {
    let response = McpClientHttpOAuthTokenResponse {
        access_token: "access-secret-token".to_string(),
        token_type: "Bearer".to_string(),
        expires_in_seconds: Some(60),
        refresh_token: Some("refresh-secret-token".to_string()),
        scope: vec!["mcp:tools".to_string(), "api_key_scope".to_string()],
    };

    let lease =
        build_mcp_client_http_oauth_token_lease_from_exchange_response(&response, 1_000_000)
            .expect("token lease");

    let response_debug = format!("{response:?}");
    let lease_debug = format!("{lease:?}");

    assert!(response_debug.contains("mcp:tools"));
    assert!(lease_debug.contains("mcp:tools"));
    assert!(!response_debug.contains("api_key_scope"));
    assert!(!lease_debug.contains("api_key_scope"));
    assert!(!response_debug.contains("access-secret-token"));
    assert!(!lease_debug.contains("access-secret-token"));
    assert!(!response_debug.contains("refresh-secret-token"));
    assert!(!lease_debug.contains("refresh-secret-token"));
}
