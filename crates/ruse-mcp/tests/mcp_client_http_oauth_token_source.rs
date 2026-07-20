use ruse_mcp::McpClientHttpOAuthTokenSourceRef;
use serde_json::json;

#[test]
fn mcp_client_http_oauth_token_source_serializes_stable_shape() {
    let source = McpClientHttpOAuthTokenSourceRef::new(
        "desktop-keychain",
        Some("user@example.com"),
        Some("https://mcp.example.com"),
    )
    .expect("token source");

    assert_eq!(source.source_id, "desktop-keychain");
    assert_eq!(source.account_hint.as_deref(), Some("user@example.com"));
    assert_eq!(source.resource.as_deref(), Some("https://mcp.example.com"));
    assert_eq!(
        serde_json::to_value(source).expect("json"),
        json!({
            "source_id": "desktop-keychain",
            "account_hint": "user@example.com",
            "resource": "https://mcp.example.com"
        })
    );
}

#[test]
fn mcp_client_http_oauth_token_source_normalizes_empty_optional_hints() {
    let source =
        McpClientHttpOAuthTokenSourceRef::new("env", Some("  "), Some("")).expect("token source");

    assert_eq!(source.account_hint, None);
    assert_eq!(source.resource, None);
}

#[test]
fn mcp_client_http_oauth_token_source_rejects_invalid_source_id_without_leaking() {
    for source_id in ["", "  ", "bad source", "mcp.token.secret"] {
        let error =
            McpClientHttpOAuthTokenSourceRef::new(source_id, None::<String>, None::<String>)
                .expect_err("invalid source");
        let message = error.to_string();

        assert!(message.contains("invalid MCP client HTTP OAuth token source"));
        assert!(!message.contains("mcp.token.secret"));
        assert!(!message.contains("token.secret"));
    }
}

#[test]
fn mcp_client_http_oauth_token_source_rejects_secret_like_hints_without_leaking() {
    let error = McpClientHttpOAuthTokenSourceRef::new(
        "desktop-keychain",
        Some("account-secret-token"),
        Some("https://mcp.example.com"),
    )
    .expect_err("invalid hint");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth token source"));
    assert!(!message.contains("account-secret-token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_token_source_rejects_oversized_hints_without_leaking() {
    let oversized = "a".repeat(300);
    let error = McpClientHttpOAuthTokenSourceRef::new(
        "desktop-keychain",
        Some(oversized),
        Some("https://mcp.example.com"),
    )
    .expect_err("oversized hint");

    assert!(
        error
            .to_string()
            .contains("invalid MCP client HTTP OAuth token source")
    );
}
