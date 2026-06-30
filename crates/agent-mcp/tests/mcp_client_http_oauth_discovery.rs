use agent_mcp::{
    parse_mcp_client_http_oauth_metadata_json,
    parse_mcp_client_http_protected_resource_metadata_json,
};
use serde_json::json;

#[test]
fn mcp_client_http_oauth_discovery_parses_authorization_server_metadata() {
    let value = json!({
        "issuer": "https://auth.example.com",
        "authorization_endpoint": "https://auth.example.com/authorize",
        "token_endpoint": "https://auth.example.com/token",
        "registration_endpoint": "https://auth.example.com/register",
        "scopes_supported": ["mcp:tools", "mcp:resources"],
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "none"],
        "unknown_field": {"ignored": true}
    });

    let metadata = parse_mcp_client_http_oauth_metadata_json(&value).expect("oauth metadata");

    assert_eq!(metadata.issuer.as_deref(), Some("https://auth.example.com"));
    assert_eq!(
        metadata.authorization_endpoint.as_deref(),
        Some("https://auth.example.com/authorize")
    );
    assert_eq!(
        metadata.token_endpoint.as_deref(),
        Some("https://auth.example.com/token")
    );
    assert_eq!(
        metadata.registration_endpoint.as_deref(),
        Some("https://auth.example.com/register")
    );
    assert_eq!(metadata.scopes_supported, ["mcp:tools", "mcp:resources"]);
    assert_eq!(
        metadata.token_endpoint_auth_methods_supported,
        ["client_secret_basic", "none"]
    );
}

#[test]
fn mcp_client_http_oauth_discovery_parses_missing_optional_fields_as_empty() {
    let value = json!({});

    let metadata = parse_mcp_client_http_oauth_metadata_json(&value).expect("oauth metadata");

    assert!(metadata.issuer.is_none());
    assert!(metadata.authorization_endpoint.is_none());
    assert!(metadata.token_endpoint.is_none());
    assert!(metadata.registration_endpoint.is_none());
    assert!(metadata.scopes_supported.is_empty());
    assert!(metadata.token_endpoint_auth_methods_supported.is_empty());
}

#[test]
fn mcp_client_http_oauth_discovery_rejects_invalid_endpoint_without_leaking_body() {
    let value = json!({
        "issuer": "https://auth.example.com",
        "token_endpoint": "file:///tmp/secret-token",
        "scopes_supported": ["mcp:tools"]
    });

    let error = parse_mcp_client_http_oauth_metadata_json(&value).expect_err("invalid endpoint");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth metadata"));
    assert!(!message.contains("file:///tmp/secret-token"));
    assert!(!message.contains("secret-token"));
}

#[test]
fn mcp_client_http_oauth_discovery_rejects_non_string_arrays_without_leaking_body() {
    let value = json!({
        "issuer": "https://auth.example.com",
        "scopes_supported": ["mcp:tools", {"secret": "token"}]
    });

    let error = parse_mcp_client_http_oauth_metadata_json(&value).expect_err("invalid scope");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth metadata"));
    assert!(!message.contains("secret"));
    assert!(!message.contains("token"));
}

#[test]
fn mcp_client_http_oauth_discovery_parses_protected_resource_metadata() {
    let value = json!({
        "resource": "https://mcp.example.com",
        "authorization_servers": ["https://auth.example.com"],
        "scopes_supported": ["mcp:tools"],
        "ignored": "yes"
    });

    let metadata =
        parse_mcp_client_http_protected_resource_metadata_json(&value).expect("resource metadata");

    assert_eq!(
        metadata.resource.as_deref(),
        Some("https://mcp.example.com")
    );
    assert_eq!(metadata.authorization_servers, ["https://auth.example.com"]);
    assert_eq!(metadata.scopes_supported, ["mcp:tools"]);
}

#[test]
fn mcp_client_http_oauth_discovery_rejects_invalid_protected_resource_without_leaking_body() {
    let value = json!({
        "resource": "https://mcp.example.com",
        "authorization_servers": ["ssh://secret-token@example.com"],
        "scopes_supported": ["mcp:tools"]
    });

    let error = parse_mcp_client_http_protected_resource_metadata_json(&value)
        .expect_err("invalid protected resource metadata");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP protected resource metadata"));
    assert!(!message.contains("ssh://secret-token@example.com"));
    assert!(!message.contains("secret-token"));
}
