use std::sync::Arc;

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentLoopBudget, AgentLoopTerminalState, AgentRuntimeConfig,
    ComprehensiveTurnInput, ComprehensiveTurnRunner, ConfigLayer, ConfigLayerKind, ConfigOverride,
    RuntimeIds, RuntimeModelConfig, build_effective_runtime_config,
};
use agent_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelEndpointConfig, ModelFallbackChain,
    ModelInvocationPolicy, ModelProviderCatalog, ModelProviderConfig, ModelProviderKind,
    ModelProviderRegistry, ModelRequestRequirements, ModelRetryPolicy, RegisteredModelProvider,
};
use agent_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};
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
        max_context_tokens: Some(4096),
    }
}

fn requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: true,
        structured_outputs: false,
        min_context_tokens: Some(1024),
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

fn effective_config(config: ModelProviderConfig) -> agent_core::EffectiveRuntimeConfig {
    let layer = ConfigLayer {
        kind: ConfigLayerKind::BuiltIn,
        source: "test".to_string(),
        config: AgentRuntimeConfig {
            schema_version: AGENT_CONFIG_SCHEMA_VERSION,
            model: RuntimeModelConfig {
                default_provider: "fake".to_string(),
                catalog: ModelProviderCatalog::new(vec![config]).expect("valid catalog"),
                invocation_policy: invocation_policy(),
            },
        },
    };

    build_effective_runtime_config(vec![layer], ConfigOverride::default())
        .expect("effective config")
}

fn loop_budget() -> AgentLoopBudget {
    AgentLoopBudget {
        max_model_turns: 4,
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
        effective_config: effective_config(config),
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
        agent_core::ConfiguredTurnError::InvalidInput { field, .. } if field == "user_message"
    ));
}
