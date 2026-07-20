use ruse_mcp::{
    FixtureMcpClient, MCP_SAMPLING_CREATE_MESSAGE_METHOD, McpClientCallbackKind,
    McpClientCallbackOutcome, McpClientCallbackPhase, McpClientCallbackTransportKind,
    McpCreateMessageResult, McpInitializeResult, McpJsonRpcFailure, McpJsonRpcSuccess, McpRoot,
    McpRootListResult, McpSamplingContent, McpSamplingRole, McpServerCapabilities, McpServerConfig,
    McpServerId, McpTransportConfig, handle_mcp_client_callback_frame,
};
use serde_json::json;

fn server_id() -> McpServerId {
    McpServerId::new("callback-server").expect("server id")
}

fn config() -> McpServerConfig {
    McpServerConfig::new(
        server_id(),
        "Callback Server",
        McpTransportConfig::stdio("mcp-callback"),
    )
    .expect("config")
}

fn init(server_id: McpServerId) -> McpInitializeResult {
    McpInitializeResult::new(
        server_id,
        "2025-11-25",
        "callback",
        Some("1.0.0".to_string()),
        McpServerCapabilities {
            tools: true,
            prompts: false,
            resources: false,
        },
    )
}

fn callback_client() -> FixtureMcpClient {
    let server_id = server_id();
    FixtureMcpClient::new().with_server(server_id.clone(), init(server_id))
}

#[tokio::test]
async fn mcp_client_reentrant_callbacks_handles_roots_list() {
    let server_id = server_id();
    let roots =
        McpRootListResult::from_roots([
            McpRoot::new("file:///workspace/project", Some("Project")).expect("root")
        ]);
    let client = callback_client().with_roots(server_id, roots);

    let dispatch = handle_mcp_client_callback_frame(
        &client,
        &client,
        &config(),
        McpClientCallbackTransportKind::Stdio,
        McpClientCallbackPhase::ToolCallInFlight,
        &json!({
            "jsonrpc": "2.0",
            "id": "roots-1",
            "method": "roots/list",
            "params": {}
        })
        .to_string(),
    )
    .await;

    assert_eq!(
        dispatch.report.callback_kind,
        McpClientCallbackKind::RootsList
    );
    assert_eq!(dispatch.report.outcome, McpClientCallbackOutcome::Responded);
    assert!(dispatch.report.response_emitted);
    let response = serde_json::from_str::<McpJsonRpcSuccess>(
        dispatch
            .response_frame
            .as_ref()
            .expect("response")
            .trim_end(),
    )
    .expect("success response");
    assert_eq!(
        response.result["roots"][0]["uri"],
        "file:///workspace/project"
    );
}

#[tokio::test]
async fn mcp_client_reentrant_callbacks_handles_sampling_without_leaking_prompt_in_report() {
    let server_id = server_id();
    let client = callback_client().with_sampling_result(
        server_id,
        McpCreateMessageResult::new(
            McpSamplingRole::Assistant,
            McpSamplingContent::text("fixture answer"),
            "fixture-model",
            Some("endTurn"),
        )
        .expect("sampling result"),
    );
    let prompt = "summarize sk-secret-token please";

    let dispatch = handle_mcp_client_callback_frame(
        &client,
        &client,
        &config(),
        McpClientCallbackTransportKind::StreamableHttp,
        McpClientCallbackPhase::ToolCallInFlight,
        &json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": MCP_SAMPLING_CREATE_MESSAGE_METHOD,
            "params": {
                "messages": [
                    {
                        "role": "user",
                        "content": {"type": "text", "text": prompt}
                    }
                ],
                "maxTokens": 64
            }
        })
        .to_string(),
    )
    .await;

    assert_eq!(
        dispatch.report.callback_kind,
        McpClientCallbackKind::SamplingCreateMessage
    );
    assert_eq!(dispatch.report.outcome, McpClientCallbackOutcome::Responded);
    assert!(dispatch.report.response_emitted);
    assert_eq!(client.sampling_call_count("callback-server"), 1);
    let report_json = serde_json::to_string(&dispatch.report).expect("report json");
    assert!(!report_json.contains(prompt));
    assert!(!report_json.contains("sk-secret-token"));
}

#[tokio::test]
async fn mcp_client_reentrant_callbacks_rejects_unknown_methods_safely() {
    let client = callback_client();

    let dispatch = handle_mcp_client_callback_frame(
        &client,
        &client,
        &config(),
        McpClientCallbackTransportKind::Stdio,
        McpClientCallbackPhase::ToolCallInFlight,
        &json!({
            "jsonrpc": "2.0",
            "id": "unknown-1",
            "method": "secrets/read",
            "params": {"token": "sk-secret-token"}
        })
        .to_string(),
    )
    .await;

    assert_eq!(
        dispatch.report.callback_kind,
        McpClientCallbackKind::Unknown
    );
    assert_eq!(dispatch.report.outcome, McpClientCallbackOutcome::Rejected);
    assert_eq!(client.sampling_call_count("callback-server"), 0);
    let response = serde_json::from_str::<McpJsonRpcFailure>(
        dispatch
            .response_frame
            .as_ref()
            .expect("response")
            .trim_end(),
    )
    .expect("failure response");
    assert_eq!(response.error.code, -32601);
    let report_json = serde_json::to_string(&dispatch.report).expect("report json");
    assert!(!report_json.contains("sk-secret-token"));
}

#[tokio::test]
async fn mcp_client_reentrant_callbacks_rejects_unsafe_sampling_before_provider() {
    let client = callback_client();
    let frame = json!({
        "jsonrpc": "2.0",
        "id": "sample-tools",
        "method": MCP_SAMPLING_CREATE_MESSAGE_METHOD,
        "params": {
            "messages": [
                {
                    "role": "user",
                    "content": {"type": "text", "text": "run a tool with api_key sk-secret"}
                }
            ],
            "maxTokens": 64,
            "tools": [{"name": "local.shell"}]
        }
    })
    .to_string();

    let dispatch = handle_mcp_client_callback_frame(
        &client,
        &client,
        &config(),
        McpClientCallbackTransportKind::Stdio,
        McpClientCallbackPhase::ToolCallInFlight,
        &frame,
    )
    .await;

    assert_eq!(
        dispatch.report.callback_kind,
        McpClientCallbackKind::SamplingCreateMessage
    );
    assert_eq!(dispatch.report.outcome, McpClientCallbackOutcome::Rejected);
    assert_eq!(client.sampling_call_count("callback-server"), 0);
    let response = serde_json::from_str::<McpJsonRpcFailure>(
        dispatch
            .response_frame
            .as_ref()
            .expect("response")
            .trim_end(),
    )
    .expect("failure response");
    assert_eq!(response.error.code, -32600);
    assert!(
        response
            .error
            .message
            .contains("unsupported_sampling_tools")
    );
    let report_json = serde_json::to_string(&dispatch.report).expect("report json");
    assert!(!report_json.contains("sk-secret"));
}

#[tokio::test]
async fn mcp_client_reentrant_callbacks_ignores_notifications_without_response() {
    let client = callback_client();

    let dispatch = handle_mcp_client_callback_frame(
        &client,
        &client,
        &config(),
        McpClientCallbackTransportKind::Stdio,
        McpClientCallbackPhase::Fixture,
        &json!({
            "jsonrpc": "2.0",
            "method": "roots/list",
            "params": {}
        })
        .to_string(),
    )
    .await;

    assert_eq!(
        dispatch.report.outcome,
        McpClientCallbackOutcome::IgnoredNotification
    );
    assert!(dispatch.response_frame.is_none());
    assert!(!dispatch.report.response_emitted);
}
