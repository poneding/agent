use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentRuntimeConfig, ConfigLayer, ConfigLayerKind, ConfigOverride,
    ConfiguredTurnError, ConfiguredTurnInput, ConfiguredTurnRunner, ModelRuntimeError, RuntimeIds,
    RuntimeModelConfig, build_effective_runtime_config,
};
use agent_model::{
    ModelCapabilities, ModelEndpointConfig, ModelError, ModelFallbackChain, ModelInvocationPolicy,
    ModelProvider, ModelProviderCatalog, ModelProviderConfig, ModelProviderKind,
    ModelProviderRegistry, ModelRequest, ModelRequestRequirements, ModelRetryPolicy,
    ModelStreamEvent, ModelStreamResult, RegisteredModelProvider,
};
use agent_types::{
    ConversationId, CorrelationId, RuntimeEventPayload, TurnId, TurnState, WorkspaceId,
};
use async_trait::async_trait;

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

fn capabilities() -> ModelCapabilities {
    ModelCapabilities {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        max_context_tokens: Some(4096),
    }
}

fn requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        min_context_tokens: Some(1024),
    }
}

fn provider_config(id: &str, model: &str) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Fake,
        model,
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid provider config")
}

fn invocation_policy() -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(1, Vec::new()).expect("valid retry"),
        ModelFallbackChain::new(Vec::<String>::new()).expect("valid fallback"),
    )
    .expect("valid policy")
}

fn runtime_config(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
) -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        schema_version: AGENT_CONFIG_SCHEMA_VERSION,
        model: RuntimeModelConfig {
            default_provider: default_provider.to_string(),
            catalog: ModelProviderCatalog::new(providers).expect("valid catalog"),
            invocation_policy: invocation_policy(),
        },
    }
}

fn effective_config(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
) -> agent_core::EffectiveRuntimeConfig {
    let layer = ConfigLayer {
        kind: ConfigLayerKind::BuiltIn,
        source: "test".to_string(),
        config: runtime_config(default_provider, providers),
    };
    build_effective_runtime_config(vec![layer], ConfigOverride::default())
        .expect("effective config")
}

fn configured_input(
    user_message: &str,
    effective_config: agent_core::EffectiveRuntimeConfig,
) -> ConfiguredTurnInput {
    ConfiguredTurnInput {
        ids: runtime_ids(),
        user_message: user_message.to_string(),
        effective_config,
        requirements: requirements(),
    }
}

fn registered(
    config: ModelProviderConfig,
    provider: Arc<dyn ModelProvider>,
) -> RegisteredModelProvider {
    RegisteredModelProvider::new(config, capabilities(), provider).expect("registered provider")
}

fn registry(providers: Vec<RegisteredModelProvider>) -> ModelProviderRegistry {
    ModelProviderRegistry::new(providers).expect("registry")
}

fn stream_text(text: &str) -> ModelStreamResult {
    vec![
        ModelStreamEvent::Started {
            request_id: format!("request-{text}"),
        },
        ModelStreamEvent::Delta {
            text: text.to_string(),
        },
        ModelStreamEvent::Completed {
            finish_reason: "stop".to_string(),
            output_token_count: text.len() as u32,
        },
    ]
}

struct RecordingProvider {
    outcomes: Mutex<VecDeque<Result<ModelStreamResult, ModelError>>>,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}

impl RecordingProvider {
    fn scripted(
        outcomes: Vec<Result<ModelStreamResult, ModelError>>,
    ) -> (Arc<Self>, Arc<Mutex<Vec<ModelRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let provider = Arc::new(Self {
            outcomes: Mutex::new(outcomes.into()),
            requests: Arc::clone(&requests),
        });
        (provider, requests)
    }
}

#[async_trait]
impl ModelProvider for RecordingProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelStreamResult, ModelError> {
        self.requests.lock().expect("requests lock").push(request);
        self.outcomes
            .lock()
            .expect("outcomes lock")
            .pop_front()
            .unwrap_or_else(|| Err(ModelError::Script("script exhausted".to_string())))
    }
}

#[tokio::test]
async fn configured_turn_runner_uses_effective_provider_and_model() {
    let config = provider_config("primary", "primary-model");
    let effective = effective_config("primary", vec![config.clone()]);
    let (provider, requests) = RecordingProvider::scripted(vec![Ok(stream_text("ready"))]);
    let registry = registry(vec![registered(config, provider)]);

    let outcome = ConfiguredTurnRunner::run(configured_input("say hello", effective), &registry)
        .await
        .expect("configured turn");

    assert_eq!(outcome.selected_provider, "primary");
    assert_eq!(outcome.selected_model, "primary-model");
    assert_eq!(outcome.final_message, "ready");
    assert_eq!(outcome.attempts.len(), 1);

    let requests = requests.lock().expect("requests");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].provider, "primary");
    assert_eq!(requests[0].model, "primary-model");
}

#[tokio::test]
async fn configured_turn_runner_rejects_empty_user_message() {
    let config = provider_config("primary", "primary-model");
    let effective = effective_config("primary", vec![config.clone()]);
    let (provider, requests) = RecordingProvider::scripted(vec![Ok(stream_text("unused"))]);
    let registry = registry(vec![registered(config, provider)]);

    let error = ConfiguredTurnRunner::run(configured_input("  ", effective), &registry)
        .await
        .expect_err("empty message rejected");

    assert!(matches!(
        error,
        ConfiguredTurnError::InvalidInput { field, .. } if field == "user_message"
    ));
    assert!(requests.lock().expect("requests").is_empty());
}

#[tokio::test]
async fn configured_turn_runner_resolution_failure_prevents_model_call() {
    let selected_config = provider_config("primary", "primary-model");
    let unused_config = provider_config("unused", "unused-model");
    let effective = effective_config("primary", vec![selected_config]);
    let (unused_provider, unused_requests) =
        RecordingProvider::scripted(vec![Ok(stream_text("unused"))]);
    let registry = registry(vec![registered(unused_config, unused_provider)]);

    let error = ConfiguredTurnRunner::run(configured_input("say hello", effective), &registry)
        .await
        .expect_err("resolution failure");

    assert!(matches!(
        error,
        ConfiguredTurnError::Runtime(
            ModelRuntimeError::ProviderNotRegistered { provider_id }
        ) if provider_id == "primary"
    ));
    assert!(unused_requests.lock().expect("requests").is_empty());
}

#[tokio::test]
async fn configured_turn_runner_success_emits_final_response() {
    let config = provider_config("primary", "primary-model");
    let effective = effective_config("primary", vec![config.clone()]);
    let (provider, _requests) = RecordingProvider::scripted(vec![Ok(stream_text("done"))]);
    let registry = registry(vec![registered(config, provider)]);

    let outcome = ConfiguredTurnRunner::run(configured_input("finish", effective), &registry)
        .await
        .expect("configured turn");

    assert!(outcome.events.iter().any(|event| matches!(
        &event.payload,
        RuntimeEventPayload::FinalResponseProduced(final_response)
            if final_response.message == "done"
    )));
    assert!(matches!(
        outcome.events.last().map(|event| &event.payload),
        Some(RuntimeEventPayload::TurnCompleted(completed))
            if completed.final_state == TurnState::Done
    ));
}
