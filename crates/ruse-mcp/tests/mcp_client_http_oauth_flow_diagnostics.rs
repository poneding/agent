use ruse_mcp::{
    McpClientHttpOAuthAuthorizationRequestInput, McpClientHttpOAuthCallbackInput,
    McpClientHttpOAuthFlowDiagnostic, McpClientHttpOAuthFlowDiagnosticCode,
    McpClientHttpOAuthFlowDiagnosticSeverity, McpClientHttpOAuthMetadata,
    McpClientHttpOAuthPkceChallenge, McpClientHttpOAuthPkceVerifier, McpClientHttpOAuthState,
    build_mcp_client_http_oauth_authorization_flow_report,
    build_mcp_client_http_oauth_authorization_request_plan,
    build_mcp_client_http_oauth_token_exchange_flow_report,
    build_mcp_client_http_oauth_token_exchange_request, validate_mcp_client_http_oauth_callback,
};
use serde_json::json;

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

fn pkce_challenge() -> McpClientHttpOAuthPkceChallenge {
    McpClientHttpOAuthPkceChallenge::s256(&verifier()).expect("challenge")
}

#[test]
fn mcp_client_http_oauth_flow_diagnostics_serialize_stable_shape() {
    let diagnostic = McpClientHttpOAuthFlowDiagnostic::error(
        McpClientHttpOAuthFlowDiagnosticCode::StateMismatch,
        "MCP HTTP OAuth callback state did not match",
    );

    assert_eq!(
        serde_json::to_value(diagnostic).expect("json"),
        json!({
            "severity": "error",
            "code": "state_mismatch",
            "message": "MCP HTTP OAuth callback state did not match"
        })
    );
}

#[test]
fn mcp_client_http_oauth_flow_diagnostics_reports_authorization_ready_without_secrets() {
    let plan = build_mcp_client_http_oauth_authorization_request_plan(
        &oauth_metadata(),
        &request_input(),
        &pkce_challenge(),
        &state(),
    )
    .expect("plan");
    let report = build_mcp_client_http_oauth_authorization_flow_report(&plan);

    assert!(report.ready);
    assert_eq!(
        report.diagnostics[0].code,
        McpClientHttpOAuthFlowDiagnosticCode::AuthorizationRequestReady
    );
    let rendered = serde_json::to_string(&report).expect("json");
    assert!(!rendered.contains("state-0123456789abcdef"));
    assert!(!rendered.contains("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"));
}

#[test]
fn mcp_client_http_oauth_flow_diagnostics_reports_callback_success() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        Some("callback-code-secret-value"),
        Some("state-0123456789abcdef"),
        None::<String>,
        None::<String>,
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");

    assert_eq!(
        validation.diagnostics[0].severity,
        McpClientHttpOAuthFlowDiagnosticSeverity::Info
    );
    assert_eq!(
        validation.diagnostics[0].code,
        McpClientHttpOAuthFlowDiagnosticCode::CallbackAccepted
    );
    let rendered = serde_json::to_string(&validation.diagnostics).expect("json");
    assert!(!rendered.contains("callback-code-secret-value"));
}

#[test]
fn mcp_client_http_oauth_flow_diagnostics_reports_invalid_callback_without_secrets() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        None::<String>,
        Some("state-0123456789abcdef"),
        None::<String>,
        None::<String>,
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");

    assert_eq!(
        validation.diagnostics[0].severity,
        McpClientHttpOAuthFlowDiagnosticSeverity::Error
    );
    assert_eq!(
        validation.diagnostics[0].code,
        McpClientHttpOAuthFlowDiagnosticCode::InvalidCallback
    );
    let rendered = serde_json::to_string(&validation.diagnostics).expect("json");
    assert!(!rendered.contains("state-0123456789abcdef"));
}

#[test]
fn mcp_client_http_oauth_flow_diagnostics_redacts_state_mismatch() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        Some("callback-code-secret-value"),
        Some("state-mismatch-secret-token"),
        None::<String>,
        None::<String>,
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");

    assert_eq!(
        validation.diagnostics[0].severity,
        McpClientHttpOAuthFlowDiagnosticSeverity::Error
    );
    assert_eq!(
        validation.diagnostics[0].code,
        McpClientHttpOAuthFlowDiagnosticCode::StateMismatch
    );
    let rendered = serde_json::to_string(&validation.diagnostics).expect("json");
    assert!(!rendered.contains("state-mismatch-secret-token"));
    assert!(!rendered.contains("callback-code-secret-value"));
    assert!(!rendered.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_flow_diagnostics_redacts_provider_error() {
    let callback = McpClientHttpOAuthCallbackInput::new(
        None::<String>,
        Some("state-0123456789abcdef"),
        Some("access_denied"),
        Some("provider secret-token description"),
    )
    .expect("callback");
    let validation =
        validate_mcp_client_http_oauth_callback(&callback, &state()).expect("validation");

    assert_eq!(
        validation.diagnostics[0].severity,
        McpClientHttpOAuthFlowDiagnosticSeverity::Error
    );
    assert_eq!(
        validation.diagnostics[0].code,
        McpClientHttpOAuthFlowDiagnosticCode::ProviderError
    );
    let rendered = serde_json::to_string(&validation.diagnostics).expect("json");
    assert!(!rendered.contains("provider secret-token description"));
    assert!(!rendered.contains("access_denied"));
    assert!(!rendered.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_flow_diagnostics_reports_token_exchange_ready_without_secrets() {
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
    .expect("request");
    let report = build_mcp_client_http_oauth_token_exchange_flow_report(&request);

    assert!(report.ready);
    assert_eq!(
        report.diagnostics[0].code,
        McpClientHttpOAuthFlowDiagnosticCode::TokenExchangeReady
    );
    let rendered = serde_json::to_string(&report).expect("json");
    assert!(!rendered.contains("callback-code-secret-value"));
    assert!(!rendered.contains("dBjftJeZ4CVP"));
}
