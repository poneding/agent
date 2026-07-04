use std::sync::Arc;

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentRuntimeConfig, ConfigLayer, ConfigLayerKind, ConfigOverride,
    ConfiguredTurnInput, ConfiguredTurnRunner, RuntimeIds, RuntimeModelConfig,
    build_effective_runtime_config,
};
use agent_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelEndpointConfig, ModelError,
    ModelEventStream, ModelFallbackChain, ModelInvocationPolicy, ModelProvider,
    ModelProviderCatalog, ModelProviderConfig, ModelProviderKind, ModelProviderRegistry,
    ModelRequest, ModelRequestRequirements, ModelRetryPolicy, RegisteredModelProvider,
};
use agent_storage::{EventStore, InMemoryEventStore, ReplayCursor};
use agent_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
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

fn effective_config(provider: ModelProviderConfig) -> agent_core::EffectiveRuntimeConfig {
    let config = AgentRuntimeConfig {
        schema_version: AGENT_CONFIG_SCHEMA_VERSION,
        model: RuntimeModelConfig {
            default_provider: provider.id.clone(),
            catalog: ModelProviderCatalog::new(vec![provider]).expect("valid catalog"),
            invocation_policy: ModelInvocationPolicy::new(
                30_000,
                ModelRetryPolicy::new(1, Vec::new()).expect("valid retry"),
                ModelFallbackChain::new(Vec::<String>::new()).expect("valid fallback"),
            )
            .expect("valid policy"),
        },
    };
    build_effective_runtime_config(
        vec![ConfigLayer {
            kind: ConfigLayerKind::BuiltIn,
            source: "test".to_string(),
            config,
        }],
        ConfigOverride::default(),
    )
    .expect("effective config")
}

fn requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        image_input: false,
        min_context_tokens: Some(1024),
        min_output_tokens: Some(1024),
    }
}

fn capabilities() -> ModelCapabilities {
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

fn configured_input(ids: RuntimeIds, provider: ModelProviderConfig) -> ConfiguredTurnInput {
    ConfiguredTurnInput {
        ids,
        user_message: "persist this turn".to_string(),
        effective_config: effective_config(provider),
        requirements: requirements(),
    }
}

fn registry(provider: ModelProviderConfig) -> ModelProviderRegistry {
    ModelProviderRegistry::new(vec![
        RegisteredModelProvider::new(
            provider,
            capabilities(),
            Arc::new(FakeModelProvider::new(FakeModelScript::text(
                "persisted response",
            ))),
        )
        .expect("registered provider"),
    ])
    .expect("registry")
}

#[derive(Debug)]
struct FailingModelProvider;

#[async_trait::async_trait]
impl ModelProvider for FailingModelProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        Err(ModelError::Provider {
            code: "test.provider_failed".to_string(),
            message: "provider failed".to_string(),
            retryable: false,
        })
    }
}

fn failing_registry(provider: ModelProviderConfig) -> ModelProviderRegistry {
    ModelProviderRegistry::new(vec![
        RegisteredModelProvider::new(provider, capabilities(), Arc::new(FailingModelProvider))
            .expect("registered provider"),
    ])
    .expect("registry")
}

#[tokio::test]
async fn configured_turn_persistence_appends_events_in_order() {
    let ids = runtime_ids();
    let provider = provider_config("fake", "fake-scripted");
    let registry = registry(provider.clone());
    let mut store = InMemoryEventStore::default();

    let outcome = ConfiguredTurnRunner::run_and_persist(
        configured_input(ids, provider),
        &registry,
        &mut store,
    )
    .await
    .expect("persist configured turn");

    let stored = store
        .list_by_conversation(ids.conversation_id)
        .expect("list persisted events");

    assert_eq!(stored.len(), outcome.events.len());
    for (index, stored_event) in stored.iter().enumerate() {
        assert_eq!(
            stored_event.event.seq.expect("sequence").get(),
            index as u64 + 1
        );
        assert_eq!(stored_event.event.id, outcome.events[index].id);
    }
}

#[tokio::test]
async fn configured_turn_persistence_replays_final_response() {
    let ids = runtime_ids();
    let provider = provider_config("fake", "fake-scripted");
    let registry = registry(provider.clone());
    let mut store = InMemoryEventStore::default();

    ConfiguredTurnRunner::run_and_persist(configured_input(ids, provider), &registry, &mut store)
        .await
        .expect("persist configured turn");

    let replayed = ReplayCursor::new(ids.conversation_id)
        .collect(&store)
        .expect("replay conversation");
    let final_response = replayed
        .iter()
        .filter_map(|event| match &event.event.payload {
            RuntimeEventPayload::FinalResponseProduced(final_response) => {
                Some(final_response.message.as_str())
            }
            _ => None,
        })
        .next_back();
    let persisted_json = store
        .list_by_conversation(ids.conversation_id)
        .expect("list persisted events")
        .iter()
        .map(|stored| serde_json::to_string(&stored.event).expect("serialize event"))
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(final_response, Some("persisted response"));
    assert!(!persisted_json.contains("sk-test-secret"));
    assert!(!persisted_json.contains("api_key"));
    assert!(!persisted_json.contains("password"));
}

#[tokio::test]
async fn configured_turn_persistence_records_prompt_snapshot_before_model_stream() {
    let ids = runtime_ids();
    let provider = provider_config("fake", "fake-scripted");
    let registry = registry(provider.clone());
    let mut store = InMemoryEventStore::default();

    ConfiguredTurnRunner::run_and_persist(configured_input(ids, provider), &registry, &mut store)
        .await
        .expect("persist configured turn");

    let stored = store
        .list_by_conversation(ids.conversation_id)
        .expect("list persisted events");
    let snapshot_index = stored
        .iter()
        .position(|stored| {
            matches!(
                stored.event.payload,
                RuntimeEventPayload::PromptSnapshotRecorded(_)
            )
        })
        .expect("prompt snapshot event exists");
    let model_stream_index = stored
        .iter()
        .position(|stored| {
            matches!(
                stored.event.payload,
                RuntimeEventPayload::ModelStreamStarted(_)
            )
        })
        .expect("model stream event exists");

    assert!(snapshot_index < model_stream_index);
}

#[tokio::test]
async fn configured_turn_persistence_keeps_prompt_snapshot_when_model_fails() {
    let ids = runtime_ids();
    let provider = provider_config("fake", "fake-scripted");
    let registry = failing_registry(provider.clone());
    let mut store = InMemoryEventStore::default();

    ConfiguredTurnRunner::run_and_persist(configured_input(ids, provider), &registry, &mut store)
        .await
        .expect_err("model failure is returned");

    let stored = store
        .list_by_conversation(ids.conversation_id)
        .expect("list persisted events");

    assert!(
        stored.iter().any(|stored| {
            matches!(
                stored.event.payload,
                RuntimeEventPayload::PromptSnapshotRecorded(_)
            )
        }),
        "prompt snapshot event should remain persisted"
    );
    assert!(
        stored
            .iter()
            .any(|stored| matches!(stored.event.payload, RuntimeEventPayload::TurnFailed(_))),
        "turn failure event should also be persisted"
    );
}
