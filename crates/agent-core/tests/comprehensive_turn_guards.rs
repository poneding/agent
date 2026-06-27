use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentLoopBudget, AgentLoopFailureReason, AgentLoopTerminalState,
    AgentRuntimeConfig, ComprehensiveTurnInput, ComprehensiveTurnResources,
    ComprehensiveTurnRunner, ConfigLayer, ConfigLayerKind, ConfigOverride, RuntimeIds,
    RuntimeModelConfig, build_effective_runtime_config,
};
use agent_model::{
    ModelCapabilities, ModelEndpointConfig, ModelError, ModelEventStream, ModelFallbackChain,
    ModelInvocationPolicy, ModelProvider, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, ModelProviderRegistry, ModelRequest, ModelRequestRequirements,
    ModelRetryPolicy, ModelStreamEvent, ModelStreamResult, RegisteredModelProvider,
    model_events_to_stream,
};
use agent_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use agent_sandbox::WorkspaceSandbox;
use agent_tool::ToolRegistry;
use agent_tool::builtins::ReadFileTool;
use agent_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};
use async_trait::async_trait;
use serde_json::json;
use tempfile::TempDir;

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

fn input(config: ModelProviderConfig) -> ComprehensiveTurnInput {
    ComprehensiveTurnInput {
        ids: runtime_ids(),
        user_message: "read README".to_string(),
        effective_config: effective_config(config),
        requirements: requirements(),
        loop_budget: loop_budget(),
    }
}

fn read_file_registry() -> (TempDir, ToolRegistry) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace dir");
    fs::write(root.join("README.md"), "hello from tool\n").expect("readme");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    let mut registry = ToolRegistry::default();
    registry
        .register(ReadFileTool::new(sandbox))
        .expect("register read tool");
    (temp, registry)
}

fn policy() -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "workspace.read_file").expect("scope"),
        PolicyDecision::Allow {
            reason: "read-only fixture".to_string(),
        },
    )
}

fn tool_call_stream(call_id: &str) -> ModelStreamResult {
    vec![
        ModelStreamEvent::Started {
            request_id: format!("request-{call_id}"),
        },
        ModelStreamEvent::ToolCall {
            call_id: call_id.to_string(),
            tool_name: "workspace.read_file".to_string(),
            input_json: json!({ "path": "README.md" }),
        },
        ModelStreamEvent::Completed {
            finish_reason: "tool_calls".to_string(),
            output_token_count: 0,
        },
    ]
}

fn final_stream() -> ModelStreamResult {
    vec![
        ModelStreamEvent::Started {
            request_id: "request-final".to_string(),
        },
        ModelStreamEvent::Delta {
            text: "done".to_string(),
        },
        ModelStreamEvent::Completed {
            finish_reason: "stop".to_string(),
            output_token_count: 4,
        },
    ]
}

struct RecordingProvider {
    streams: Mutex<VecDeque<ModelStreamResult>>,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}

impl RecordingProvider {
    fn scripted(streams: Vec<ModelStreamResult>) -> (Arc<Self>, Arc<Mutex<Vec<ModelRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let provider = Arc::new(Self {
            streams: Mutex::new(streams.into()),
            requests: Arc::clone(&requests),
        });
        (provider, requests)
    }
}

#[async_trait]
impl ModelProvider for RecordingProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        self.requests.lock().expect("requests").push(request);
        let stream = self
            .streams
            .lock()
            .expect("streams")
            .pop_front()
            .ok_or_else(|| ModelError::Script("script exhausted".to_string()))?;
        Ok(model_events_to_stream(stream))
    }
}

fn registry(
    config: ModelProviderConfig,
    provider: Arc<dyn ModelProvider>,
) -> ModelProviderRegistry {
    ModelProviderRegistry::new(vec![
        RegisteredModelProvider::new(config, capabilities(), provider)
            .expect("registered provider"),
    ])
    .expect("registry")
}

#[tokio::test]
async fn comprehensive_turn_rejects_invalid_loop_budget_before_model_call() {
    let config = provider_config();
    let mut input = input(config.clone());
    input.loop_budget.max_model_turns = 0;
    let (provider, requests) = RecordingProvider::scripted(vec![final_stream()]);
    let models = registry(config, provider);
    let (_temp, tools) = read_file_registry();
    let policy = policy();

    let outcome = ComprehensiveTurnRunner::run_with_resources(
        input,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
    )
    .await
    .expect("guarded turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Failed);
    assert_eq!(
        outcome.failure_reason,
        Some(AgentLoopFailureReason::InvalidBudget)
    );
    assert!(requests.lock().expect("requests").is_empty());
}

#[tokio::test]
async fn comprehensive_turn_fails_when_model_turn_budget_is_exhausted() {
    let config = provider_config();
    let mut input = input(config.clone());
    input.loop_budget.max_model_turns = 1;
    let (provider, requests) =
        RecordingProvider::scripted(vec![tool_call_stream("call-1"), final_stream()]);
    let models = registry(config, provider);
    let (_temp, tools) = read_file_registry();
    let policy = policy();

    let outcome = ComprehensiveTurnRunner::run_with_resources(
        input,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
    )
    .await
    .expect("guarded turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Failed);
    assert_eq!(
        outcome.failure_reason,
        Some(AgentLoopFailureReason::MaxModelTurnsExceeded)
    );
    assert_eq!(requests.lock().expect("requests").len(), 1);
    assert!(
        outcome
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::RuntimeError(_)))
    );
}

#[tokio::test]
async fn comprehensive_turn_fails_before_executing_tool_over_budget() {
    let config = provider_config();
    let mut input = input(config.clone());
    input.loop_budget.max_tool_calls = 1;
    let (provider, _requests) = RecordingProvider::scripted(vec![
        tool_call_stream("call-1"),
        tool_call_stream("call-2"),
        final_stream(),
    ]);
    let models = registry(config, provider);
    let (_temp, tools) = read_file_registry();
    let policy = policy();

    let outcome = ComprehensiveTurnRunner::run_with_resources(
        input,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
    )
    .await
    .expect("guarded turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Failed);
    assert_eq!(
        outcome.failure_reason,
        Some(AgentLoopFailureReason::MaxToolCallsExceeded)
    );
    let started = outcome
        .events
        .iter()
        .filter(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionStarted(_)))
        .count();
    assert_eq!(started, 1);
}
