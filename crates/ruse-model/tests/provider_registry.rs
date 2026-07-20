use std::sync::Arc;

use ruse_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelCapability, ModelEndpointConfig,
    ModelMessage, ModelProviderConfig, ModelProviderKind, ModelProviderRegistry,
    ModelRegistryError, ModelRequest, ModelRequestRequirements, ModelStreamEvent,
    RegisteredModelProvider, collect_model_event_stream,
};
use ruse_types::{ConversationId, CorrelationId, TurnId};

fn fake_config(id: &str) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Fake,
        "fake-scripted",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid fake config")
}

fn fake_capabilities() -> ModelCapabilities {
    ModelCapabilities {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        image_input: false,
        max_context_tokens: Some(4096),
        max_output_tokens: Some(1024),
        cost: None,
    }
}

fn fake_registered(id: &str, capabilities: ModelCapabilities) -> RegisteredModelProvider {
    RegisteredModelProvider::new(
        fake_config(id),
        capabilities,
        Arc::new(FakeModelProvider::new(FakeModelScript::text("ready"))),
    )
    .expect("valid registered provider")
}

fn model_request() -> ModelRequest {
    ModelRequest {
        provider: "fake".to_string(),
        model: "fake-scripted".to_string(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
        messages: vec![ModelMessage::user("hello")],
        reasoning_effort: None,
        tools: Vec::new(),
    }
}

#[tokio::test]
async fn provider_registry_registers_and_resolves_provider() {
    let registry = ModelProviderRegistry::new(vec![fake_registered("fake", fake_capabilities())])
        .expect("valid registry");

    let registered = registry.get("fake").expect("registered provider");
    let stream = registered
        .provider()
        .stream(model_request())
        .await
        .expect("stream fake provider");
    let events = collect_model_event_stream(stream)
        .await
        .expect("collect stream");

    assert_eq!(registered.config.id, "fake");
    assert_eq!(registered.capabilities, fake_capabilities());
    assert_eq!(
        events,
        vec![
            ModelStreamEvent::Started {
                request_id: "fake-request-1".to_string()
            },
            ModelStreamEvent::Delta {
                text: "ready".to_string()
            },
            ModelStreamEvent::Completed {
                finish_reason: "stop".to_string(),
                output_token_count: 5
            }
        ]
    );
}

#[test]
fn provider_registry_rejects_duplicate_provider_ids() {
    let err = ModelProviderRegistry::new(vec![
        fake_registered("fake", fake_capabilities()),
        fake_registered("fake", fake_capabilities()),
    ])
    .expect_err("duplicate provider rejected");

    assert!(matches!(
        err,
        ModelRegistryError::DuplicateProviderId { id } if id == "fake"
    ));
}

#[test]
fn provider_registry_reports_provider_not_found() {
    let registry = ModelProviderRegistry::new(vec![fake_registered("fake", fake_capabilities())])
        .expect("valid registry");

    let err = registry
        .get("missing")
        .expect_err("missing provider rejected");

    assert!(matches!(
        err,
        ModelRegistryError::ProviderNotFound { id } if id == "missing"
    ));
}

#[test]
fn provider_registry_accepts_compatible_capabilities() {
    let registry = ModelProviderRegistry::new(vec![fake_registered("fake", fake_capabilities())])
        .expect("valid registry");
    let requirements = ModelRequestRequirements {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        image_input: false,
        min_context_tokens: Some(2048),
        min_output_tokens: Some(512),
    };

    let registered = registry
        .require_capabilities("fake", &requirements)
        .expect("compatible provider");

    assert_eq!(registered.config.id, "fake");
}

#[test]
fn provider_registry_reports_missing_capabilities() {
    let registry = ModelProviderRegistry::new(vec![fake_registered("fake", fake_capabilities())])
        .expect("valid registry");
    let requirements = ModelRequestRequirements {
        streaming: true,
        tool_calls: true,
        structured_outputs: true,
        image_input: true,
        min_context_tokens: Some(8192),
        min_output_tokens: Some(2048),
    };

    let err = registry
        .require_capabilities("fake", &requirements)
        .expect_err("missing capabilities rejected");

    assert!(matches!(
        err,
        ModelRegistryError::MissingCapabilities { provider_id, missing }
            if provider_id == "fake"
                && missing == vec![
                    ModelCapability::ToolCalls,
                    ModelCapability::StructuredOutputs,
                    ModelCapability::ImageInput,
                    ModelCapability::ContextTokens,
                    ModelCapability::OutputTokens,
                ]
    ));
}
