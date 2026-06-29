use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use agent_core::{
    McpModelSamplingProvider, McpSamplingModelBridgeConfig, McpSamplingModelBridgeIds,
};
use agent_mcp::{
    McpCreateMessageRequestParams, McpError, McpSamplingContent, McpSamplingMessage,
    McpSamplingProvider as _, McpSamplingRole, McpServerId,
};
use agent_model::{
    FakeModelProvider, FakeModelScript, ModelError, ModelEventStream, ModelProvider, ModelRequest,
    ModelStreamEvent, model_events_to_stream,
};
use agent_types::{ConversationId, CorrelationId, TurnId};
use async_trait::async_trait;

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

fn request(text: &str) -> McpCreateMessageRequestParams {
    McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(McpSamplingContent::text(text))],
        128,
    )
}

#[tokio::test]
async fn mcp_sampling_model_provider_returns_fake_model_text() {
    let provider = McpModelSamplingProvider::new(
        Arc::new(FakeModelProvider::new(FakeModelScript::text(
            "sampled answer",
        ))),
        bridge_config(),
        bridge_ids(),
    );

    let result = provider
        .create_message(&server_id(), request("please answer"))
        .await
        .expect("sampling succeeds");

    assert_eq!(result.role, McpSamplingRole::Assistant);
    assert_eq!(result.model, "local-model");
    assert_eq!(result.stop_reason.as_deref(), Some("stop"));
    assert_eq!(
        result.content.content,
        [McpSamplingContent::text("sampled answer")]
    );
}

#[tokio::test]
async fn mcp_sampling_model_provider_returns_empty_text_for_empty_deltas() {
    let provider = McpModelSamplingProvider::new(
        Arc::new(FakeModelProvider::new(FakeModelScript {
            request_id: "empty-response".to_string(),
            deltas: Vec::new(),
            tool_calls: Vec::new(),
            finish_reason: "stop".to_string(),
        })),
        bridge_config(),
        bridge_ids(),
    );

    let result = provider
        .create_message(&server_id(), request("please answer"))
        .await
        .expect("sampling succeeds");

    assert_eq!(result.content.content, [McpSamplingContent::text("")]);
}

#[tokio::test]
async fn mcp_sampling_model_provider_rejects_invalid_request_before_streaming() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = McpModelSamplingProvider::new(
        Arc::new(CountingModelProvider {
            calls: Arc::clone(&calls),
        }),
        bridge_config(),
        bridge_ids(),
    );

    let error = provider
        .create_message(
            &server_id(),
            McpCreateMessageRequestParams::new(Vec::new(), 128),
        )
        .await
        .expect_err("invalid request is rejected");

    assert!(matches!(error, McpError::InvalidServerRequest(_)));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn mcp_sampling_model_provider_debug_does_not_retain_prompt_text() {
    let secret_prompt = "api_key=sk-provider-secret";
    let provider = McpModelSamplingProvider::new(
        Arc::new(FakeModelProvider::new(FakeModelScript::text(
            "sampled answer",
        ))),
        bridge_config(),
        bridge_ids(),
    );

    provider
        .create_message(&server_id(), request(secret_prompt))
        .await
        .expect("sampling succeeds");

    let debug_text = format!("{provider:?}");
    assert!(!debug_text.contains(secret_prompt));
    assert!(!debug_text.contains("sk-provider-secret"));
}

struct CountingModelProvider {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl ModelProvider for CountingModelProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(model_events_to_stream(vec![
            ModelStreamEvent::Started {
                request_id: "counting-provider".to_string(),
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 0,
            },
        ]))
    }
}
