use agent_mcp::{McpClientHttpOAuthMetadata, McpClientHttpProtectedResourceMetadata};

#[test]
fn mcp_client_http_oauth_metadata_accepts_valid_contracts() {
    let metadata = McpClientHttpOAuthMetadata {
        issuer: Some("https://auth.example.com".to_string()),
        authorization_endpoint: Some("https://auth.example.com/authorize".to_string()),
        token_endpoint: Some("https://auth.example.com/token".to_string()),
        registration_endpoint: Some("https://auth.example.com/register".to_string()),
        scopes_supported: vec!["mcp:tools".to_string(), "mcp:resources".to_string()],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string(),
            "none".to_string(),
        ],
    };
    let resource = McpClientHttpProtectedResourceMetadata {
        resource: Some("https://mcp.example.com".to_string()),
        authorization_servers: vec!["https://auth.example.com".to_string()],
        scopes_supported: vec!["mcp:tools".to_string()],
    };

    metadata.validate().expect("valid oauth metadata");
    resource
        .validate()
        .expect("valid protected resource metadata");
}

#[test]
fn mcp_client_http_oauth_metadata_rejects_invalid_urls_without_leaking() {
    let metadata = McpClientHttpOAuthMetadata {
        issuer: Some("postgres://user:secret-token@example.com/db".to_string()),
        authorization_endpoint: Some("https://auth.example.com/authorize".to_string()),
        token_endpoint: Some("https://auth.example.com/token".to_string()),
        registration_endpoint: None,
        scopes_supported: vec!["mcp:tools".to_string()],
        token_endpoint_auth_methods_supported: Vec::new(),
    };

    let error = metadata.validate().expect_err("invalid issuer");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth metadata"));
    assert!(!message.contains("postgres"));
    assert!(!message.contains("secret-token"));
    assert!(!message.contains("user"));
}

#[test]
fn mcp_client_http_oauth_metadata_rejects_invalid_scopes_without_leaking() {
    let metadata = McpClientHttpOAuthMetadata {
        issuer: Some("https://auth.example.com".to_string()),
        authorization_endpoint: None,
        token_endpoint: Some("https://auth.example.com/token".to_string()),
        registration_endpoint: None,
        scopes_supported: vec!["mcp:tools".to_string(), "bad scope token".to_string()],
        token_endpoint_auth_methods_supported: Vec::new(),
    };

    let error = metadata.validate().expect_err("invalid scope");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP OAuth metadata"));
    assert!(!message.contains("bad scope token"));
}

#[test]
fn mcp_client_http_oauth_metadata_rejects_invalid_auth_methods_without_leaking() {
    let metadata = McpClientHttpOAuthMetadata {
        issuer: Some("https://auth.example.com".to_string()),
        authorization_endpoint: None,
        token_endpoint: Some("https://auth.example.com/token".to_string()),
        registration_endpoint: None,
        scopes_supported: vec!["mcp:tools".to_string()],
        token_endpoint_auth_methods_supported: vec!["".to_string()],
    };

    let error = metadata.validate().expect_err("invalid auth method");

    assert!(
        error
            .to_string()
            .contains("invalid MCP client HTTP OAuth metadata")
    );
}

#[test]
fn mcp_client_http_oauth_metadata_protected_resource_rejects_invalid_authorization_server() {
    let resource = McpClientHttpProtectedResourceMetadata {
        resource: Some("https://mcp.example.com".to_string()),
        authorization_servers: vec!["file:///tmp/auth".to_string()],
        scopes_supported: vec!["mcp:tools".to_string()],
    };

    let error = resource.validate().expect_err("invalid auth server");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP protected resource metadata"));
    assert!(!message.contains("file:///tmp/auth"));
}

#[test]
fn mcp_client_http_oauth_metadata_serializes_stable_shape() {
    let metadata = McpClientHttpOAuthMetadata {
        issuer: Some("https://auth.example.com".to_string()),
        authorization_endpoint: Some("https://auth.example.com/authorize".to_string()),
        token_endpoint: Some("https://auth.example.com/token".to_string()),
        registration_endpoint: None,
        scopes_supported: vec!["mcp:tools".to_string()],
        token_endpoint_auth_methods_supported: vec!["none".to_string()],
    };
    let value = serde_json::to_value(metadata).expect("json");

    assert_eq!(value["issuer"], "https://auth.example.com");
    assert_eq!(
        value["authorization_endpoint"],
        "https://auth.example.com/authorize"
    );
    assert_eq!(value["token_endpoint"], "https://auth.example.com/token");
    assert!(value["registration_endpoint"].is_null());
    assert_eq!(value["scopes_supported"][0], "mcp:tools");
    assert_eq!(value["token_endpoint_auth_methods_supported"][0], "none");
}

#[test]
fn mcp_client_http_oauth_metadata_protected_resource_serializes_stable_shape() {
    let metadata = McpClientHttpProtectedResourceMetadata {
        resource: Some("https://mcp.example.com".to_string()),
        authorization_servers: vec!["https://auth.example.com".to_string()],
        scopes_supported: vec!["mcp:tools".to_string()],
    };
    let value = serde_json::to_value(metadata).expect("json");

    assert_eq!(value["resource"], "https://mcp.example.com");
    assert_eq!(
        value["authorization_servers"][0],
        "https://auth.example.com"
    );
    assert_eq!(value["scopes_supported"][0], "mcp:tools");
}
