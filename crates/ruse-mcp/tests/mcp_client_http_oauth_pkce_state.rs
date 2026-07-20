use ruse_mcp::{
    McpClientHttpOAuthPkceChallenge, McpClientHttpOAuthPkceVerifier, McpClientHttpOAuthState,
};
use serde_json::json;

#[test]
fn mcp_client_http_oauth_pkce_state_accepts_valid_contracts() {
    let verifier =
        McpClientHttpOAuthPkceVerifier::new("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
            .expect("verifier");
    let state = McpClientHttpOAuthState::new("state-0123456789abcdef").expect("state");

    assert_eq!(
        verifier.verifier,
        "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
    );
    assert_eq!(state.state, "state-0123456789abcdef");
}

#[test]
fn mcp_client_http_oauth_pkce_state_builds_s256_known_vector() {
    let verifier =
        McpClientHttpOAuthPkceVerifier::new("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
            .expect("verifier");
    let challenge = McpClientHttpOAuthPkceChallenge::s256(&verifier).expect("challenge");

    assert_eq!(challenge.method, "S256");
    assert_eq!(
        challenge.challenge,
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
    assert_eq!(
        serde_json::to_value(&challenge).expect("json"),
        json!({
            "challenge": "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
            "method": "S256"
        })
    );
}

#[test]
fn mcp_client_http_oauth_pkce_state_rejects_invalid_verifiers_without_leaking() {
    let error =
        McpClientHttpOAuthPkceVerifier::new("short-secret-token").expect_err("invalid verifier");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth flow"));
    assert!(!message.contains("short-secret-token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_pkce_state_rejects_invalid_state_without_leaking() {
    let error = McpClientHttpOAuthState::new("state secret-token").expect_err("invalid state");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth flow"));
    assert!(!message.contains("state secret-token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_pkce_state_debug_is_redacted() {
    let verifier =
        McpClientHttpOAuthPkceVerifier::new("debugVerifierValue012345678901234567890123456789")
            .expect("verifier");
    let state = McpClientHttpOAuthState::new("debug-state-0123456789").expect("state");

    let verifier_debug = format!("{verifier:?}");
    let state_debug = format!("{state:?}");

    assert!(verifier_debug.contains("[redacted]"));
    assert!(state_debug.contains("[redacted]"));
    assert!(!verifier_debug.contains("debugVerifierValue"));
    assert!(!state_debug.contains("debug-state"));
}
