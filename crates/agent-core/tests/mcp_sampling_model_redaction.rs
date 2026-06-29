use std::sync::Arc;

use agent_core::{
    McpModelSamplingProvider, McpSamplingModelBridgeConfig, McpSamplingModelBridgeError,
    McpSamplingModelBridgeIds, build_mcp_sampling_model_request,
    mcp_sampling_bridge_error_diagnostic, mcp_sampling_model_error_diagnostic,
};
use agent_mcp::{
    McpCreateMessageRequestParams, McpSamplingContent, McpSamplingMessage,
    McpSamplingProvider as _, McpServerId,
};
use agent_model::{ModelError, ModelEventStream, ModelProvider, ModelRequest};
use agent_types::{ConversationId, CorrelationId, TurnId};
use async_trait::async_trait;
use serde_json::json;

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

fn server_id() -> McpServerId {
    McpServerId::new("fixture-server").expect("valid server id")
}

fn text_request(text: &str) -> McpCreateMessageRequestParams {
    McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(McpSamplingContent::text(text))],
        128,
    )
}

#[test]
fn mcp_sampling_model_redaction_hides_prompt_text_in_mapping_errors() {
    let secret_prompt = "api_key=sk-redaction-secret";
    let request = McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(
            McpSamplingContent::image("c2VjcmV0LWltYWdl", "image/png").expect("valid image"),
        )],
        128,
    )
    .with_system_prompt(secret_prompt);

    let error = build_mcp_sampling_model_request(&request, &bridge_config(), bridge_ids())
        .expect_err("image content is unsupported");
    let diagnostic = mcp_sampling_bridge_error_diagnostic(&error);
    let public_text = format!("{error} {diagnostic:?}");

    assert_eq!(diagnostic.code, "unsupported_content");
    assert!(!public_text.contains(secret_prompt));
    assert!(!public_text.contains("sk-redaction-secret"));
    assert!(!public_text.contains("c2VjcmV0LWltYWdl"));
}

#[tokio::test]
async fn mcp_sampling_model_redaction_hides_provider_error_secrets() {
    let provider = McpModelSamplingProvider::new(
        Arc::new(ErroringModelProvider),
        bridge_config(),
        bridge_ids(),
    );

    let error = provider
        .create_message(&server_id(), text_request("hello"))
        .await
        .expect_err("provider failure is redacted");
    let error_text = error.to_string();

    assert!(error_text.contains("model_provider_failed"));
    assert!(!error_text.contains("sk-provider-secret"));
    assert!(!error_text.contains("api_key"));
    assert!(!error_text.contains("raw provider response"));
}

#[test]
fn mcp_sampling_model_redaction_bounds_public_messages() {
    let long_value = format!("{}{}", "safe-code-".repeat(80), "sk-tail-secret");
    let diagnostic = mcp_sampling_bridge_error_diagnostic(
        &McpSamplingModelBridgeError::InvalidRequest(long_value),
    );

    assert_eq!(diagnostic.code, "invalid_request");
    assert!(diagnostic.message.chars().count() <= 256);
    assert!(!diagnostic.message.contains("sk-tail-secret"));
}

#[test]
fn mcp_sampling_model_redaction_serializes_stable_diagnostic_shape() {
    let diagnostic = mcp_sampling_model_error_diagnostic(&ModelError::Provider {
        code: "rate_limit".to_string(),
        message: "raw provider response with api_key=sk-provider-secret".to_string(),
        retryable: true,
    });

    let value = serde_json::to_value(diagnostic).expect("diagnostic serializes");
    assert_eq!(
        value,
        json!({
            "code": "model_provider_failed",
            "message": "sampling model provider failed: rate_limit"
        })
    );
}

struct ErroringModelProvider;

#[async_trait]
impl ModelProvider for ErroringModelProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        Err(ModelError::Provider {
            code: "sk-provider-secret".to_string(),
            message: "raw provider response with api_key=sk-provider-secret".to_string(),
            retryable: false,
        })
    }
}
