use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};

use ruse_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentLoopBudget, AgentLoopTerminalState, AgentRuntimeConfig,
    ComprehensiveTurnInput, ComprehensiveTurnResources, ComprehensiveTurnRunner, ConfigLayer,
    ConfigLayerKind, ConfigOverride, RuntimeIds, RuntimeModelConfig,
    build_effective_runtime_config,
};
use ruse_model::{
    ModelCapabilities, ModelEndpointConfig, ModelError, ModelEventStream, ModelFallbackChain,
    ModelInvocationPolicy, ModelProvider, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, ModelProviderRegistry, ModelRequest, ModelRequestRequirements,
    ModelRetryPolicy, ModelStreamEvent, ModelStreamResult, RegisteredModelProvider,
    model_events_to_stream,
};
use ruse_policy::{
    CapabilityAction, PolicyDecision, ResourceKind, ResourceScope, StaticPolicyEngine,
};
use ruse_sandbox::WorkspaceSandbox;
use ruse_storage::InMemoryEventStore;
use ruse_tool::ToolRegistry;
use ruse_tool::builtins::FileEditTool;
use ruse_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};
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

fn invocation_policy() -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(1, Vec::new()).expect("valid retry"),
        ModelFallbackChain::new(Vec::<String>::new()).expect("valid fallback"),
    )
    .expect("valid policy")
}

fn effective_config(config: ModelProviderConfig) -> ruse_core::EffectiveRuntimeConfig {
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
            terminal: Default::default(),
        },
    };

    build_effective_runtime_config(vec![layer], ConfigOverride::default())
        .expect("effective config")
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

fn input(ids: RuntimeIds, config: ModelProviderConfig) -> ComprehensiveTurnInput {
    ComprehensiveTurnInput {
        ids,
        user_message: "edit README".to_string(),
        conversation_history: std::sync::Arc::new(Vec::new()),
        effective_config: std::sync::Arc::new(effective_config(config)),
        requirements: requirements(),
        loop_budget: loop_budget(),
    }
}

fn edit_registry() -> (TempDir, ToolRegistry) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("workspace");
    fs::create_dir(&root).expect("workspace dir");
    fs::write(root.join("README.md"), "old text\n").expect("readme");
    let sandbox = WorkspaceSandbox::new(&root).expect("sandbox");
    let mut registry = ToolRegistry::default();
    registry
        .register(FileEditTool::new(sandbox))
        .expect("register edit tool");
    (temp, registry)
}

fn policy() -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Execute,
        ResourceScope::new(ResourceKind::Tool, "workspace.edit_file").expect("scope"),
        PolicyDecision::Allow {
            reason: "allow edit fixture".to_string(),
        },
    )
}

fn edit_stream() -> ModelStreamResult {
    vec![
        ModelStreamEvent::Started {
            request_id: "request-edit".to_string(),
        },
        ModelStreamEvent::ToolCall {
            call_id: "call-edit".to_string(),
            tool_name: "workspace.edit_file".to_string(),
            input_json: json!({
                "path": "README.md",
                "expected": "old text\n",
                "replacement": "new text\n",
                "apply": true
            }),
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
            text: "edited".to_string(),
        },
        ModelStreamEvent::Completed {
            finish_reason: "stop".to_string(),
            output_token_count: 6,
        },
    ]
}

struct ScriptedProvider {
    streams: Mutex<VecDeque<ModelStreamResult>>,
}

impl ScriptedProvider {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            streams: Mutex::new(vec![edit_stream(), final_stream()].into()),
        })
    }
}

#[async_trait]
impl ModelProvider for ScriptedProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        let stream = self
            .streams
            .lock()
            .expect("streams")
            .pop_front()
            .ok_or_else(|| ModelError::Script("script exhausted".to_string()))?;
        Ok(model_events_to_stream(stream))
    }
}

fn registry(config: ModelProviderConfig) -> ModelProviderRegistry {
    ModelProviderRegistry::new(vec![
        RegisteredModelProvider::new(config, capabilities(), ScriptedProvider::new())
            .expect("registered provider"),
    ])
    .expect("registry")
}

#[tokio::test]
async fn comprehensive_turn_replays_completed_tool_effect_without_reexecution() {
    let ids = runtime_ids();
    let config = provider_config();
    let (temp, tools) = edit_registry();
    let policy = policy();
    let mut effects = InMemoryEventStore::default();

    let first = ComprehensiveTurnRunner::run_with_idempotency(
        input(ids, config.clone()),
        &registry(config.clone()),
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
        &mut effects,
    )
    .await
    .expect("first turn");
    assert_eq!(first.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "new text\n"
    );

    fs::write(temp.path().join("workspace/README.md"), "old text\n").expect("reset file");

    let second = ComprehensiveTurnRunner::run_with_idempotency(
        input(ids, config.clone()),
        &registry(config),
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
        &mut effects,
    )
    .await
    .expect("second turn");

    assert_eq!(second.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(
        fs::read_to_string(temp.path().join("workspace/README.md")).expect("read file"),
        "old text\n",
        "replayed completed effect must not mutate the file again"
    );
    assert!(second.events.iter().any(|event| matches!(
        event.payload,
        RuntimeEventPayload::ToolObservationRecorded(_)
    )));
}
