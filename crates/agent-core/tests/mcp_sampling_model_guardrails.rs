use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use agent_core::{
    McpModelSamplingProvider, McpSamplingModelBridgeConfig, McpSamplingModelBridgeIds,
};
use agent_mcp::{
    McpCreateMessageRequestParams, McpError, McpSamplingContent, McpSamplingIncludeContext,
    McpSamplingMessage, McpSamplingProvider as _, McpServerId,
};
use agent_model::{
    FakeModelProvider, FakeModelScript, ModelError, ModelEventStream, ModelProvider, ModelRequest,
    ModelStreamEvent, model_events_to_stream,
};
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

#[tokio::test]
async fn mcp_sampling_model_guardrails_reject_tools_before_streaming() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = counting_sampling_provider(Arc::clone(&calls));
    let mut request = text_request("hello");
    request.tools.push(json!({
        "name": "dangerous_tool",
        "inputSchema": {"type": "object"}
    }));

    let error = provider
        .create_message(&server_id(), request)
        .await
        .expect_err("server-requested tools are rejected");

    assert_invalid_request(error, "unsupported_sampling_tools");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn mcp_sampling_model_guardrails_reject_context_before_streaming() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = counting_sampling_provider(Arc::clone(&calls));
    let request = text_request("hello").with_include_context(McpSamplingIncludeContext::ThisServer);

    let error = provider
        .create_message(&server_id(), request)
        .await
        .expect_err("context inclusion is rejected");

    assert_invalid_request(error, "unsupported_context");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn mcp_sampling_model_guardrails_reject_model_tool_call_without_input_leak() {
    let provider = McpModelSamplingProvider::new(
        Arc::new(FakeModelProvider::new(FakeModelScript::tool_call(
            "call-1",
            "dangerous_tool",
            json!({"api_key": "sk-tool-secret"}),
        ))),
        bridge_config(),
        bridge_ids(),
    );

    let error = provider
        .create_message(&server_id(), text_request("hello"))
        .await
        .expect_err("model tool calls are unsupported");
    let error_text = error.to_string();

    assert!(matches!(error, McpError::InvalidServerRequest(_)));
    assert!(error_text.contains("unsupported_content"));
    assert!(error_text.contains("model_tool_call"));
    assert!(!error_text.contains("dangerous_tool"));
    assert!(!error_text.contains("api_key"));
    assert!(!error_text.contains("sk-tool-secret"));
}

#[tokio::test]
async fn mcp_sampling_model_guardrails_reject_media_before_streaming() {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = counting_sampling_provider(Arc::clone(&calls));
    let request = McpCreateMessageRequestParams::new(
        vec![McpSamplingMessage::user(
            McpSamplingContent::audio("YXVkaW8=", "audio/wav").expect("valid audio"),
        )],
        128,
    );

    let error = provider
        .create_message(&server_id(), request)
        .await
        .expect_err("media conversion is unsupported");

    assert_invalid_request(error, "unsupported_content");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

fn counting_sampling_provider(calls: Arc<AtomicUsize>) -> McpModelSamplingProvider {
    McpModelSamplingProvider::new(
        Arc::new(CountingModelProvider { calls }),
        bridge_config(),
        bridge_ids(),
    )
}

fn assert_invalid_request(error: McpError, expected_code: &str) {
    let error_text = error.to_string();
    assert!(matches!(error, McpError::InvalidServerRequest(_)));
    assert!(
        error_text.contains(expected_code),
        "expected {error_text:?} to contain {expected_code:?}"
    );
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
