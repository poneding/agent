use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentLoopBudget, AgentLoopTerminalState, AgentRuntimeConfig,
    ComprehensiveTurnInput, ComprehensiveTurnResources, ComprehensiveTurnRunner, ConfigLayer,
    ConfigLayerKind, ConfigOverride, RuntimeIds, RuntimeModelConfig,
    build_effective_runtime_config,
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
        user_message: "read README and summarize".to_string(),
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

fn policy(decision: PolicyDecision) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "workspace.read_file").expect("scope"),
        decision,
    )
}

fn tool_call_stream() -> ModelStreamResult {
    vec![
        ModelStreamEvent::Started {
            request_id: "request-1".to_string(),
        },
        ModelStreamEvent::ToolCall {
            call_id: "call-1".to_string(),
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
            request_id: "request-2".to_string(),
        },
        ModelStreamEvent::Delta {
            text: "read complete".to_string(),
        },
        ModelStreamEvent::Completed {
            finish_reason: "stop".to_string(),
            output_token_count: 13,
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
async fn comprehensive_turn_feeds_tool_observation_to_next_model_turn() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, requests) =
        RecordingProvider::scripted(vec![tool_call_stream(), final_stream()]);
    let models = registry(config, provider);
    let (_temp, tools) = read_file_registry();
    let policy = policy(PolicyDecision::Allow {
        reason: "read-only fixture".to_string(),
    });

    let outcome = ComprehensiveTurnRunner::run_with_resources(
        input,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
    )
    .await
    .expect("turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(outcome.final_message.as_deref(), Some("read complete"));

    let requests = requests.lock().expect("requests");
    assert_eq!(requests.len(), 2);
    assert!(requests[1].messages.iter().any(|message| {
        message.contains("tool observation") && message.contains("hello from tool")
    }));
}

#[tokio::test]
async fn comprehensive_turn_suspends_when_policy_asks() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, requests) =
        RecordingProvider::scripted(vec![tool_call_stream(), final_stream()]);
    let models = registry(config, provider);
    let (_temp, tools) = read_file_registry();
    let policy = policy(PolicyDecision::Ask {
        reason: "needs approval".to_string(),
    });

    let outcome = ComprehensiveTurnRunner::run_with_resources(
        input,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
    )
    .await
    .expect("turn");

    assert_eq!(
        outcome.terminal_state,
        AgentLoopTerminalState::WaitingApproval
    );
    assert_eq!(requests.lock().expect("requests").len(), 1);
    assert!(outcome.events.iter().any(|event| matches!(
        event.payload,
        RuntimeEventPayload::PermissionDecisionRecorded(_)
    )));
    assert!(
        !outcome
            .events
            .iter()
            .any(|event| matches!(event.payload, RuntimeEventPayload::ToolExecutionStarted(_)))
    );
}
