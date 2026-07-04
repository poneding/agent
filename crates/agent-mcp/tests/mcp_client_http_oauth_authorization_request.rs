use agent_mcp::{
    McpClientHttpOAuthAuthorizationRequestInput, McpClientHttpOAuthMetadata,
    McpClientHttpOAuthPkceChallenge, McpClientHttpOAuthPkceVerifier, McpClientHttpOAuthState,
    build_mcp_client_http_oauth_authorization_request_plan,
};
use serde_json::json;

fn oauth_metadata() -> McpClientHttpOAuthMetadata {
    McpClientHttpOAuthMetadata {
        issuer: Some("https://auth.example.com".to_string()),
        authorization_endpoint: Some("https://auth.example.com/authorize".to_string()),
        token_endpoint: Some("https://auth.example.com/token".to_string()),
        registration_endpoint: None,
        scopes_supported: vec!["mcp:tools".to_string(), "mcp:resources".to_string()],
        token_endpoint_auth_methods_supported: vec!["none".to_string()],
    }
}

fn pkce_challenge() -> McpClientHttpOAuthPkceChallenge {
    let verifier =
        McpClientHttpOAuthPkceVerifier::new("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
            .expect("verifier");
    McpClientHttpOAuthPkceChallenge::s256(&verifier).expect("challenge")
}

#[test]
fn mcp_client_http_oauth_authorization_request_builds_stable_url_plan() {
    let input = McpClientHttpOAuthAuthorizationRequestInput::new(
        "mcp-client",
        "http://127.0.0.1:8080/oauth/callback",
        vec!["mcp:tools".to_string(), "mcp:resources".to_string()],
    )
    .expect("input");
    let state = McpClientHttpOAuthState::new("state-0123456789abcdef").expect("state");
    let plan = build_mcp_client_http_oauth_authorization_request_plan(
        &oauth_metadata(),
        &input,
        &pkce_challenge(),
        &state,
    )
    .expect("plan");

    assert_eq!(plan.state_id, "state-0123456789abcdef");
    assert_eq!(plan.code_challenge_method, "S256");
    assert_eq!(plan.scopes, vec!["mcp:tools", "mcp:resources"]);
    assert_eq!(
        plan.authorization_url,
        "https://auth.example.com/authorize?response_type=code&client_id=mcp-client&redirect_uri=http%3A%2F%2F127.0.0.1%3A8080%2Foauth%2Fcallback&scope=mcp%3Atools+mcp%3Aresources&state=state-0123456789abcdef&code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&code_challenge_method=S256"
    );
    assert_eq!(
        serde_json::to_value(&plan).expect("json"),
        json!({
            "authorization_url": plan.authorization_url,
            "state_id": "state-0123456789abcdef",
            "code_challenge_method": "S256",
            "scopes": ["mcp:tools", "mcp:resources"]
        })
    );
}

#[test]
fn mcp_client_http_oauth_authorization_request_rejects_missing_endpoint() {
    let mut metadata = oauth_metadata();
    metadata.authorization_endpoint = None;
    let input = McpClientHttpOAuthAuthorizationRequestInput::new(
        "mcp-client",
        "http://127.0.0.1:8080/oauth/callback",
        vec!["mcp:tools".to_string()],
    )
    .expect("input");
    let state = McpClientHttpOAuthState::new("state-0123456789abcdef").expect("state");
    let error = build_mcp_client_http_oauth_authorization_request_plan(
        &metadata,
        &input,
        &pkce_challenge(),
        &state,
    )
    .expect_err("missing endpoint");

    assert!(
        error
            .to_string()
            .contains("invalid MCP client HTTP OAuth flow")
    );
}

#[test]
fn mcp_client_http_oauth_authorization_request_rejects_invalid_input_without_leaking() {
    let error = McpClientHttpOAuthAuthorizationRequestInput::new(
        "client secret-token",
        "postgres://user:secret-token@example.com/db",
        vec!["bad scope token".to_string()],
    )
    .expect_err("invalid input");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth flow"));
    assert!(!message.contains("client secret-token"));
    assert!(!message.contains("postgres"));
    assert!(!message.contains("bad scope token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_authorization_request_does_not_leak_verifier() {
    let verifier_value = "VerifierValue0123456789012345678901234567890123";
    let verifier = McpClientHttpOAuthPkceVerifier::new(verifier_value).expect("verifier");
    let challenge = McpClientHttpOAuthPkceChallenge::s256(&verifier).expect("challenge");
    let input = McpClientHttpOAuthAuthorizationRequestInput::new(
        "mcp-client",
        "http://127.0.0.1:8080/oauth/callback",
        vec!["mcp:tools".to_string()],
    )
    .expect("input");
    let state = McpClientHttpOAuthState::new("state-0123456789abcdef").expect("state");
    let plan = build_mcp_client_http_oauth_authorization_request_plan(
        &oauth_metadata(),
        &input,
        &challenge,
        &state,
    )
    .expect("plan");
    let rendered = serde_json::to_string(&plan).expect("json");

    assert!(!rendered.contains(verifier_value));
    assert!(!rendered.contains("VerifierValue"));
}
