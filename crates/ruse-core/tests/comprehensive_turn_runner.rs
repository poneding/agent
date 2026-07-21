use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use futures_util::stream;
use ruse_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentLoopBudget, AgentLoopTerminalState, AgentRuntimeConfig,
    ComprehensiveTurnInput, ComprehensiveTurnRunner, ConfigLayer, ConfigLayerKind, ConfigOverride,
    RuntimeIds, RuntimeModelConfig, build_effective_runtime_config,
};
use ruse_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelEndpointConfig, ModelError,
    ModelEventStream, ModelFailureKind, ModelFallbackChain, ModelInvocationPolicy, ModelProvider,
    ModelProviderCatalog, ModelProviderConfig, ModelProviderKind, ModelProviderRegistry,
    ModelRequest, ModelRequestRequirements, ModelRetryPolicy, ModelStreamEvent,
    RegisteredModelProvider,
};
use ruse_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};
use serde_json::json;

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
        tool_calls: true,
        structured_outputs: false,
        image_input: false,
        max_context_tokens: Some(4096),
        max_output_tokens: Some(1024),
        cost: None,
    }
}

fn requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: true,
        structured_outputs: false,
        image_input: false,
        min_context_tokens: Some(1024),
        min_output_tokens: Some(1024),
    }
}

fn provider_config() -> ModelProviderConfig {
    ModelProviderConfig::new(
        "fake",
        ModelProviderKind::Fake,
        "fake-scripted",
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

fn invocation_policy_with_fallback(fallback_provider_ids: Vec<&str>) -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(1, vec![ModelFailureKind::Transport]).expect("valid retry"),
        ModelFallbackChain::new(fallback_provider_ids).expect("valid fallback"),
    )
    .expect("valid policy")
}

fn effective_config(config: ModelProviderConfig) -> ruse_core::EffectiveRuntimeConfig {
    effective_config_with_policy("fake", vec![config], invocation_policy())
}

fn effective_config_with_policy(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
    invocation_policy: ModelInvocationPolicy,
) -> ruse_core::EffectiveRuntimeConfig {
    let layer = ConfigLayer {
        kind: ConfigLayerKind::BuiltIn,
        source: "test".to_string(),
        config: AgentRuntimeConfig {
            schema_version: AGENT_CONFIG_SCHEMA_VERSION,
            model: RuntimeModelConfig {
                default_provider: default_provider.to_string(),
                catalog: ModelProviderCatalog::new(providers).expect("valid catalog"),
                invocation_policy,
            },
            terminal: Default::default(),
        },
    };

    build_effective_runtime_config(vec![layer], ConfigOverride::default())
        .expect("effective config")
}

fn input_with_config(
    effective_config: ruse_core::EffectiveRuntimeConfig,
    user_message: &str,
) -> ComprehensiveTurnInput {
    ComprehensiveTurnInput {
        ids: runtime_ids(),
        user_message: user_message.to_string(),
        conversation_history: std::sync::Arc::new(Vec::new()),
        effective_config: std::sync::Arc::new(effective_config),
        requirements: requirements(),
        loop_budget: loop_budget(),
    }
}

fn loop_budget() -> AgentLoopBudget {
    AgentLoopBudget {
        model_turn_checkpoint_interval: 4,
        max_tool_calls: 4,
        max_wall_time_ms: 30_000,
        max_observation_bytes: 4096,
        max_consecutive_invalid_tool_calls: 2,
    }
}

fn input(config: ModelProviderConfig, user_message: &str) -> ComprehensiveTurnInput {
    ComprehensiveTurnInput {
        ids: runtime_ids(),
        user_message: user_message.to_string(),
        conversation_history: std::sync::Arc::new(Vec::new()),
        effective_config: std::sync::Arc::new(effective_config(config)),
        requirements: requirements(),
        loop_budget: loop_budget(),
    }
}

fn registry(config: ModelProviderConfig, script: FakeModelScript) -> ModelProviderRegistry {
    let provider = FakeModelProvider::new(script);
    ModelProviderRegistry::new(vec![
        RegisteredModelProvider::new(config, capabilities(), Arc::new(provider))
            .expect("registered fake provider"),
    ])
    .expect("registry")
}

enum ProviderOutcome {
    Stream(Vec<Result<ModelStreamEvent, ModelError>>),
}

struct ScriptedProvider {
    outcomes: Mutex<VecDeque<ProviderOutcome>>,
}

impl ScriptedProvider {
    fn new(outcomes: Vec<ProviderOutcome>) -> Arc<Self> {
        Arc::new(Self {
            outcomes: Mutex::new(outcomes.into()),
        })
    }
}

#[async_trait]
impl ModelProvider for ScriptedProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        let outcome = self
            .outcomes
            .lock()
            .expect("outcomes lock")
            .pop_front()
            .expect("scripted outcome");
        match outcome {
            ProviderOutcome::Stream(events) => Ok(Box::pin(stream::iter(events))),
        }
    }
}

fn stream_text(text: &str) -> Vec<Result<ModelStreamEvent, ModelError>> {
    vec![
        Ok(ModelStreamEvent::Started {
            request_id: "request-ok".to_string(),
        }),
        Ok(ModelStreamEvent::Delta {
            text: text.to_string(),
        }),
        Ok(ModelStreamEvent::Completed {
            finish_reason: "stop".to_string(),
            output_token_count: text.len().try_into().unwrap_or(u32::MAX),
        }),
    ]
}

fn stream_transport_error() -> Vec<Result<ModelStreamEvent, ModelError>> {
    vec![
        Ok(ModelStreamEvent::Started {
            request_id: "request-primary".to_string(),
        }),
        Err(ModelError::Provider {
            code: "model.transport".to_string(),
            message: "transport failed".to_string(),
            retryable: true,
        }),
    ]
}

fn fixture_with_fake_text(text: &str) -> (ComprehensiveTurnInput, ModelProviderRegistry) {
    let config = provider_config();
    let input = input(config.clone(), "finish");
    let registry = registry(config, FakeModelScript::text(text));
    (input, registry)
}

fn fixture_with_fake_tool_call(
    call_id: &str,
    tool_name: &str,
) -> (ComprehensiveTurnInput, ModelProviderRegistry) {
    let config = provider_config();
    let input = input(config.clone(), "read a file");
    let registry = registry(
        config,
        FakeModelScript::tool_call(call_id, tool_name, json!({ "path": "README.md" })),
    );
    (input, registry)
}

#[tokio::test]
async fn comprehensive_turn_runner_finishes_on_final_response() {
    let (input, registry) = fixture_with_fake_text("done");

    let outcome = ComprehensiveTurnRunner::run(input, &registry)
        .await
        .expect("comprehensive turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(outcome.final_message.as_deref(), Some("done"));
    assert!(
        outcome
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::FinalResponseProduced(_)))
    );
    assert_eq!(outcome.attempts.len(), 1);
    assert_eq!(outcome.attempts[0].provider_id, "fake");
}

#[tokio::test]
async fn comprehensive_turn_runner_records_fallback_attempts_on_success() {
    let primary_config = ModelProviderConfig::new(
        "primary",
        ModelProviderKind::Fake,
        "primary-model",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("primary config");
    let backup_config = ModelProviderConfig::new(
        "backup",
        ModelProviderKind::Fake,
        "backup-model",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("backup config");
    let input = input_with_config(
        effective_config_with_policy(
            "primary",
            vec![primary_config.clone(), backup_config.clone()],
            invocation_policy_with_fallback(vec!["backup"]),
        ),
        "finish with fallback",
    );
    let registry = ModelProviderRegistry::new(vec![
        RegisteredModelProvider::new(
            primary_config,
            capabilities(),
            ScriptedProvider::new(vec![ProviderOutcome::Stream(stream_transport_error())]),
        )
        .expect("primary provider"),
        RegisteredModelProvider::new(
            backup_config,
            capabilities(),
            ScriptedProvider::new(vec![ProviderOutcome::Stream(stream_text("backup done"))]),
        )
        .expect("backup provider"),
    ])
    .expect("registry");

    let outcome = ComprehensiveTurnRunner::run(input, &registry)
        .await
        .expect("comprehensive fallback");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(outcome.final_message.as_deref(), Some("backup done"));
    assert_eq!(outcome.attempts.len(), 2);
    assert_eq!(outcome.attempts[0].provider_id, "primary");
    assert_eq!(outcome.attempts[1].provider_id, "backup");
    assert_eq!(
        outcome.attempts[0].request_id.as_deref(),
        Some("request-primary")
    );
    assert_eq!(
        outcome.attempts[1].request_id.as_deref(),
        Some("request-ok")
    );
    assert!(outcome.attempts[0].elapsed_ms.is_some());
    assert!(outcome.attempts[1].elapsed_ms.is_some());
}

#[tokio::test]
async fn comprehensive_turn_runner_waits_on_tool_proposal() {
    let (input, registry) = fixture_with_fake_tool_call("call-1", "workspace.read_file");

    let outcome = ComprehensiveTurnRunner::run(input, &registry)
        .await
        .expect("comprehensive turn");

    assert_eq!(
        outcome.terminal_state,
        AgentLoopTerminalState::WaitingApproval
    );
    assert!(outcome.final_message.is_none());
    assert!(
        outcome
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ToolCallProposed(_)))
    );
}

#[tokio::test]
async fn comprehensive_turn_runner_rejects_empty_user_message() {
    let (mut input, registry) = fixture_with_fake_text("unused");
    input.user_message = "  ".to_string();

    let error = ComprehensiveTurnRunner::run(input, &registry)
        .await
        .expect_err("empty message rejected");

    assert!(matches!(
        error,
        ruse_core::ConfiguredTurnError::InvalidInput { field, .. } if field == "user_message"
    ));
}
