use agent_core::{
    McpSamplingModelBridgeConfig, McpSamplingModelBridgeIds, build_mcp_sampling_model_request,
};
use agent_mcp::{
    McpCreateMessageRequestParams, McpModelHint, McpModelPreferences, McpSamplingContent,
    McpSamplingMessage,
};
use agent_model::ModelMessage;
use agent_types::{ConversationId, CorrelationId, TurnId};

fn bridge_config() -> McpSamplingModelBridgeConfig {
    McpSamplingModelBridgeConfig {
        provider: "local-provider".to_string(),
        model: "local-model".to_string(),
    }
}

fn bridge_ids() -> McpSamplingModelBridgeIds {
    McpSamplingModelBridgeIds {
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

#[test]
fn mcp_sampling_model_mapping_builds_text_model_request() {
    let ids = bridge_ids();
    let request = McpCreateMessageRequestParams::new(
        vec![
            McpSamplingMessage::user(McpSamplingContent::text("hello")),
            McpSamplingMessage::assistant(McpSamplingContent::text("previous answer")),
        ],
        128,
    );

    let mapped = build_mcp_sampling_model_request(&request, &bridge_config(), ids)
        .expect("text sampling request maps");

    assert_eq!(mapped.provider, "local-provider");
    assert_eq!(mapped.model, "local-model");
    assert_eq!(mapped.conversation_id, ids.conversation_id);
    assert_eq!(mapped.turn_id, ids.turn_id);
    assert_eq!(mapped.correlation_id, ids.correlation_id);
    assert_eq!(
        mapped.messages,
        [
            ModelMessage::user("hello"),
            ModelMessage::assistant("previous answer"),
        ]
    );
}

#[test]
fn mcp_sampling_model_mapping_includes_system_prompt() {
    let request = McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(McpSamplingContent::text(
            "summarize this",
        ))],
        128,
    )
    .with_system_prompt("answer tersely");

    let mapped = build_mcp_sampling_model_request(&request, &bridge_config(), bridge_ids())
        .expect("system prompt maps");

    assert_eq!(
        mapped.messages,
        [
            ModelMessage::system("answer tersely"),
            ModelMessage::user("summarize this"),
        ]
    );
}

#[test]
fn mcp_sampling_model_mapping_uses_local_model_over_server_hints() {
    let request = McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(McpSamplingContent::text("hello"))],
        128,
    )
    .with_model_preferences(
        McpModelPreferences::new()
            .with_hint(McpModelHint::new("server-requested-model").expect("valid hint")),
    );

    let mapped = build_mcp_sampling_model_request(&request, &bridge_config(), bridge_ids())
        .expect("model hint remains advisory");

    assert_eq!(mapped.provider, "local-provider");
    assert_eq!(mapped.model, "local-model");
}

#[test]
fn mcp_sampling_model_mapping_rejects_media_without_prompt_leak() {
    let secret_prompt = "api_key=sk-test-secret should never appear";
    let request = McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(
            McpSamplingContent::image("aW1hZ2U=", "image/png").expect("valid image content"),
        )],
        128,
    )
    .with_system_prompt(secret_prompt);

    let error = build_mcp_sampling_model_request(&request, &bridge_config(), bridge_ids())
        .expect_err("media is unsupported in M41-01");
    let error_text = error.to_string();

    assert!(error_text.contains("unsupported_content"));
    assert!(!error_text.contains(secret_prompt));
    assert!(!error_text.contains("sk-test-secret"));
    assert!(!error_text.contains("aW1hZ2U="));
}
