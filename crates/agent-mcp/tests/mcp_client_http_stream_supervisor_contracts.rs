use agent_mcp::{
    McpClientHttpReconnectPolicy, McpClientHttpStreamSupervisorConfig,
    McpClientHttpStreamSupervisorStatus, McpServerId,
};

fn server_id() -> McpServerId {
    McpServerId::new("http-stream-supervisor-contracts").expect("server id")
}

#[test]
fn mcp_client_http_stream_supervisor_contracts_default_config_is_valid_and_bounded() {
    let config = McpClientHttpStreamSupervisorConfig::new(server_id());

    config.validate().expect("valid config");

    assert_eq!(config.server_id, server_id());
    assert_eq!(
        config.reconnect_policy,
        McpClientHttpReconnectPolicy::default()
    );
    assert_eq!(config.max_ticks, 1);
}

#[test]
fn mcp_client_http_stream_supervisor_contracts_reject_invalid_tick_budget_without_leaking() {
    let config = McpClientHttpStreamSupervisorConfig {
        server_id: server_id(),
        reconnect_policy: McpClientHttpReconnectPolicy::default(),
        max_ticks: 0,
    };

    let error = config.validate().expect_err("invalid max ticks");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP stream supervisor config"));
    assert!(!message.contains("secret"));
    assert!(!message.contains("token"));
}

#[test]
fn mcp_client_http_stream_supervisor_contracts_reject_invalid_reconnect_policy() {
    let config = McpClientHttpStreamSupervisorConfig {
        server_id: server_id(),
        reconnect_policy: McpClientHttpReconnectPolicy {
            max_attempts: 0,
            initial_backoff_ms: 250,
            max_backoff_ms: 4_000,
        },
        max_ticks: 1,
    };

    let error = config.validate().expect_err("invalid policy");
    let message = error.to_string();

    assert!(message.contains("invalid MCP client HTTP reconnect policy"));
    assert!(!message.contains("secret"));
    assert!(!message.contains("token"));
}

#[test]
fn mcp_client_http_stream_supervisor_contracts_status_serializes_stable_shape() {
    let cases = [
        (McpClientHttpStreamSupervisorStatus::Idle, "idle"),
        (McpClientHttpStreamSupervisorStatus::Running, "running"),
        (
            McpClientHttpStreamSupervisorStatus::WaitingRetry,
            "waiting_retry",
        ),
        (McpClientHttpStreamSupervisorStatus::Cancelled, "cancelled"),
        (McpClientHttpStreamSupervisorStatus::Failed, "failed"),
        (McpClientHttpStreamSupervisorStatus::Completed, "completed"),
    ];

    for (status, expected) in cases {
        let value = serde_json::to_value(status).expect("json");
        assert_eq!(value, expected);
    }
}

#[test]
fn mcp_client_http_stream_supervisor_contracts_config_serializes_stable_shape() {
    let config = McpClientHttpStreamSupervisorConfig {
        server_id: server_id(),
        reconnect_policy: McpClientHttpReconnectPolicy {
            max_attempts: 4,
            initial_backoff_ms: 100,
            max_backoff_ms: 1_000,
        },
        max_ticks: 8,
    };

    let value = serde_json::to_value(config).expect("json");

    assert_eq!(value["server_id"], "http-stream-supervisor-contracts");
    assert_eq!(value["reconnect_policy"]["max_attempts"], 4);
    assert_eq!(value["reconnect_policy"]["initial_backoff_ms"], 100);
    assert_eq!(value["reconnect_policy"]["max_backoff_ms"], 1_000);
    assert_eq!(value["max_ticks"], 8);
}
