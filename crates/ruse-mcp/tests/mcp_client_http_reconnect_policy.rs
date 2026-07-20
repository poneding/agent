use ruse_mcp::{
    McpClientHttpReconnectDecision, McpClientHttpReconnectFailureKind,
    McpClientHttpReconnectPolicy, McpError, classify_mcp_client_http_reconnect_error,
};

#[test]
fn mcp_client_http_reconnect_policy_default_is_valid_and_bounded() {
    let policy = McpClientHttpReconnectPolicy::default();

    policy.validate().expect("valid policy");

    assert_eq!(policy.max_attempts, 3);
    assert_eq!(policy.initial_backoff_ms, 250);
    assert_eq!(policy.max_backoff_ms, 4_000);
}

#[test]
fn mcp_client_http_reconnect_policy_rejects_invalid_bounds_without_leaking() {
    let zero_attempts = McpClientHttpReconnectPolicy {
        max_attempts: 0,
        initial_backoff_ms: 250,
        max_backoff_ms: 4_000,
    };
    let zero_backoff = McpClientHttpReconnectPolicy {
        max_attempts: 3,
        initial_backoff_ms: 0,
        max_backoff_ms: 4_000,
    };
    let inverted_backoff = McpClientHttpReconnectPolicy {
        max_attempts: 3,
        initial_backoff_ms: 4_001,
        max_backoff_ms: 4_000,
    };

    for invalid in [zero_attempts, zero_backoff, inverted_backoff] {
        let error = invalid.validate().expect_err("invalid policy");
        let message = error.to_string();

        assert!(message.contains("invalid MCP client HTTP reconnect policy"));
        assert!(!message.contains("secret"));
        assert!(!message.contains("token"));
    }
}

#[test]
fn mcp_client_http_reconnect_policy_computes_capped_backoff() {
    let policy = McpClientHttpReconnectPolicy {
        max_attempts: 5,
        initial_backoff_ms: 100,
        max_backoff_ms: 250,
    };

    assert_eq!(
        policy.decision_for_attempt(1, true).expect("decision"),
        McpClientHttpReconnectDecision::Retry { backoff_ms: 100 }
    );
    assert_eq!(
        policy.decision_for_attempt(2, true).expect("decision"),
        McpClientHttpReconnectDecision::Retry { backoff_ms: 200 }
    );
    assert_eq!(
        policy.decision_for_attempt(3, true).expect("decision"),
        McpClientHttpReconnectDecision::Retry { backoff_ms: 250 }
    );
    assert_eq!(
        policy.decision_for_attempt(5, true).expect("decision"),
        McpClientHttpReconnectDecision::Stop
    );
    assert_eq!(
        policy.decision_for_attempt(1, false).expect("decision"),
        McpClientHttpReconnectDecision::Stop
    );
}

#[test]
fn mcp_client_http_reconnect_policy_classifies_retryable_failures_safely() {
    let transport = McpError::Client(
        "MCP client HTTP stream transport failed with sk-secret-token".to_string(),
    );
    let status =
        McpError::Client("MCP client HTTP request failed with HTTP status 503".to_string());

    let transport_kind = classify_mcp_client_http_reconnect_error(&transport);
    let status_kind = classify_mcp_client_http_reconnect_error(&status);

    assert_eq!(
        transport_kind,
        McpClientHttpReconnectFailureKind::RetryableTransport
    );
    assert_eq!(
        status_kind,
        McpClientHttpReconnectFailureKind::RetryableStatus
    );
    assert!(transport_kind.is_retryable());
    assert!(status_kind.is_retryable());
    assert!(!transport_kind.public_reason().contains("sk-secret-token"));
}

#[test]
fn mcp_client_http_reconnect_policy_classifies_non_retryable_failures() {
    let invalid_config = McpError::InvalidConfig("secret header token".to_string());
    let invalid_response = McpError::InvalidServerRequest(
        "invalid MCP client HTTP response body with sk-secret-token".to_string(),
    );

    let config_kind = classify_mcp_client_http_reconnect_error(&invalid_config);
    let response_kind = classify_mcp_client_http_reconnect_error(&invalid_response);

    assert_eq!(
        config_kind,
        McpClientHttpReconnectFailureKind::NonRetryableConfig
    );
    assert_eq!(
        response_kind,
        McpClientHttpReconnectFailureKind::NonRetryableValidation
    );
    assert!(!config_kind.is_retryable());
    assert!(!response_kind.is_retryable());
    assert!(!config_kind.public_reason().contains("secret"));
    assert!(!response_kind.public_reason().contains("sk-secret-token"));
}

#[test]
fn mcp_client_http_reconnect_policy_serializes_stable_shape() {
    let policy = McpClientHttpReconnectPolicy {
        max_attempts: 4,
        initial_backoff_ms: 125,
        max_backoff_ms: 1_000,
    };

    let value = serde_json::to_value(policy).expect("json");

    assert_eq!(value["max_attempts"], 4);
    assert_eq!(value["initial_backoff_ms"], 125);
    assert_eq!(value["max_backoff_ms"], 1_000);
}
