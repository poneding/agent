use agent_mcp::{
    McpClientHttpOAuthAuthorizationRequestInput, McpClientHttpOAuthCallbackInput,
    McpClientHttpOAuthCallbackOutcome, McpClientHttpOAuthMetadata, McpClientHttpOAuthPkceVerifier,
    McpClientHttpOAuthState, build_mcp_client_http_oauth_token_exchange_request,
    validate_mcp_client_http_oauth_callback,
};

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

fn request_input() -> McpClientHttpOAuthAuthorizationRequestInput {
    McpClientHttpOAuthAuthorizationRequestInput::new(
        "mcp-client",
        "http://127.0.0.1:8080/oauth/callback",
        vec!["mcp:tools".to_string()],
    )
    .expect("input")
}

fn verifier() -> McpClientHttpOAuthPkceVerifier {
    McpClientHttpOAuthPkceVerifier::new("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
        .expect("verifier")
}

fn state() -> McpClientHttpOAuthState {
    McpClientHttpOAuthState::new("state-0123456789abcdef").expect("state")
}

#[test]
fn mcp_client_http_oauth_callback_accepts_matching_code_and_state() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        Some("callback-code-secret-value"),
        Some("state-0123456789abcdef"),
        None::<String>,
        None::<String>,
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");

    assert!(validation.accepted);
    assert_eq!(
        validation.outcome,
        McpClientHttpOAuthCallbackOutcome::Success
    );
    assert_eq!(
        validation.code.as_deref(),
        Some("callback-code-secret-value")
    );
}

#[test]
fn mcp_client_http_oauth_callback_rejects_state_mismatch_without_leaking() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        Some("callback-code-secret-value"),
        Some("state-mismatch-secret-token"),
        None::<String>,
        None::<String>,
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");

    assert!(!validation.accepted);
    assert_eq!(
        validation.outcome,
        McpClientHttpOAuthCallbackOutcome::StateMismatch
    );
    assert!(validation.code.is_none());

    let rendered = format!("{callback:?} {validation:?}");
    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("callback-code-secret-value"));
    assert!(!rendered.contains("state-mismatch-secret-token"));
    assert!(!rendered.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_callback_reports_provider_error_without_leaking() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        None::<String>,
        Some("state-0123456789abcdef"),
        Some("access_denied"),
        Some("provider secret-token description"),
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");

    assert!(!validation.accepted);
    assert_eq!(
        validation.outcome,
        McpClientHttpOAuthCallbackOutcome::ProviderError
    );
    assert!(validation.code.is_none());

    let rendered = format!("{callback:?} {validation:?}");
    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("provider secret-token description"));
    assert!(!rendered.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_callback_builds_token_exchange_request() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        Some("callback-code-secret-value"),
        Some("state-0123456789abcdef"),
        None::<String>,
        None::<String>,
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");
    let request = build_mcp_client_http_oauth_token_exchange_request(
        &oauth_metadata(),
        &request_input(),
        &verifier(),
        &validation,
    )
    .expect("token exchange request");

    assert_eq!(request.token_endpoint, "https://auth.example.com/token");
    assert_eq!(
        request.form,
        vec![
            ("grant_type".to_string(), "authorization_code".to_string()),
            ("code".to_string(), "callback-code-secret-value".to_string()),
            (
                "redirect_uri".to_string(),
                "http://127.0.0.1:8080/oauth/callback".to_string()
            ),
            ("client_id".to_string(), "mcp-client".to_string()),
            (
                "code_verifier".to_string(),
                "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".to_string()
            )
        ]
    );

    let debug = format!("{request:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("callback-code-secret-value"));
    assert!(!debug.contains("dBjftJeZ4CVP"));
}

#[test]
fn mcp_client_http_oauth_callback_rejects_exchange_without_success() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        None::<String>,
        Some("state-0123456789abcdef"),
        Some("access_denied"),
        None::<String>,
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");
    let error = build_mcp_client_http_oauth_token_exchange_request(
        &oauth_metadata(),
        &request_input(),
        &verifier(),
        &validation,
    )
    .expect_err("not exchangeable");

    assert!(
        error
            .to_string()
            .contains("invalid MCP client HTTP OAuth flow")
    );
}
