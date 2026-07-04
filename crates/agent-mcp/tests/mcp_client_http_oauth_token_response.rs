use agent_mcp::{
    parse_mcp_client_http_oauth_token_error_response_json,
    parse_mcp_client_http_oauth_token_response_json,
};

#[test]
fn mcp_client_http_oauth_token_response_parses_bearer_success() {
    let response = parse_mcp_client_http_oauth_token_response_json(
        r#"{
            "access_token": "access-secret-token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "refresh-secret-token",
            "scope": "mcp:tools mcp:resources"
        }"#,
    )
    .expect("token response");

    assert_eq!(response.access_token, "access-secret-token");
    assert_eq!(response.token_type, "Bearer");
    assert_eq!(response.expires_in_seconds, Some(3600));
    assert_eq!(
        response.refresh_token.as_deref(),
        Some("refresh-secret-token")
    );
    assert_eq!(response.scope, ["mcp:tools", "mcp:resources"]);
}

#[test]
fn mcp_client_http_oauth_token_response_debug_is_redacted() {
    let response = parse_mcp_client_http_oauth_token_response_json(
        r#"{
            "access_token": "debug-access-secret-token",
            "token_type": "Bearer",
            "refresh_token": "debug-refresh-secret-token"
        }"#,
    )
    .expect("token response");

    let debug = format!("{response:?}");

    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("debug-access-secret-token"));
    assert!(!debug.contains("debug-refresh-secret-token"));
    assert!(!debug.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_response_rejects_invalid_without_leaking_body() {
    let error = parse_mcp_client_http_oauth_token_response_json(
        r#"{
            "access_token": "missing-type-secret-token",
            "expires_in": "soon"
        }"#,
    )
    .expect_err("invalid token response");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token response"));
    assert!(!message.contains("missing-type-secret-token"));
    assert!(!message.contains("secret-token"));
    assert!(!message.contains("soon"));
}

#[test]
fn mcp_client_http_oauth_token_response_rejects_missing_access_token_without_leaking() {
    let error = parse_mcp_client_http_oauth_token_response_json(
        r#"{
            "token_type": "Bearer",
            "refresh_token": "refresh-secret-token"
        }"#,
    )
    .expect_err("missing access token");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token response"));
    assert!(!message.contains("refresh-secret-token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_response_parses_provider_error_response() {
    let response = parse_mcp_client_http_oauth_token_error_response_json(
        r#"{
            "error": "invalid_grant",
            "error_description": "provider secret-token description"
        }"#,
    )
    .expect("provider error");

    assert_eq!(response.error, "invalid_grant");
    assert_eq!(
        response.error_description.as_deref(),
        Some("provider secret-token description")
    );

    let debug = format!("{response:?}");
    assert!(debug.contains("invalid_grant"));
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("provider secret-token description"));
    assert!(!debug.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_response_rejects_invalid_provider_error_without_leaking() {
    let error = parse_mcp_client_http_oauth_token_error_response_json(
        r#"{
            "error_description": "provider secret-token description"
        }"#,
    )
    .expect_err("invalid provider error");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token response"));
    assert!(!message.contains("provider secret-token description"));
    assert!(!message.contains("secret-token"));
}
