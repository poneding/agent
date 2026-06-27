use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::process::Command;
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
use agent_storage::InMemoryEventStore;
use agent_tool::ToolRegistry;
use agent_tool::builtins::{FileEditTool, GitStatusTool, ReadFileTool, ShellCommandTool};
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
        max_model_turns: 8,
        max_tool_calls: 8,
        max_wall_time_ms: 30_000,
        max_observation_bytes: 16_384,
        max_consecutive_invalid_tool_calls: 2,
    }
}

fn input(ids: RuntimeIds, config: ModelProviderConfig) -> ComprehensiveTurnInput {
    ComprehensiveTurnInput {
        ids,
        user_message: "Update notes.txt and verify the workspace".to_string(),
        effective_config: effective_config(config),
        requirements: requirements(),
        loop_budget: loop_budget(),
    }
}

struct LocalCodingFixture {
    workspace: TempDir,
    tools: ToolRegistry,
    policy: StaticPolicyEngine,
}

fn local_coding_fixture() -> LocalCodingFixture {
    let workspace = tempfile::tempdir().expect("tempdir");
    initialize_git_fixture(workspace.path());
    let sandbox = WorkspaceSandbox::new(workspace.path()).expect("sandbox");
    let mut tools = ToolRegistry::default();
    tools
        .register(ReadFileTool::new(sandbox.clone()))
        .expect("register read");
    tools
        .register(FileEditTool::new(sandbox.clone()))
        .expect("register edit");
    tools
        .register(ShellCommandTool::new(sandbox.clone()))
        .expect("register shell");
    tools
        .register(GitStatusTool::new(sandbox))
        .expect("register git status");

    LocalCodingFixture {
        workspace,
        tools,
        policy: allow_policy([
            "workspace.read_file",
            "workspace.edit_file",
            "workspace.shell_command",
            "workspace.git_status",
        ]),
    }
}

fn initialize_git_fixture(root: &Path) {
    run_git(root, &["init", "-q"]);
    run_git(root, &["config", "user.name", "Agent"]);
    run_git(root, &["config", "user.email", "agent@example.invalid"]);
    fs::write(root.join("notes.txt"), "hello\n").expect("write notes");
    run_git(root, &["add", "notes.txt"]);
    run_git(root, &["commit", "-m", "initial", "--quiet"]);
}

fn run_git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn allow_policy<const N: usize>(tool_names: [&str; N]) -> StaticPolicyEngine {
    let mut policy = StaticPolicyEngine::default();
    for tool_name in tool_names {
        policy = policy.with_decision(
            CapabilityAction::Read,
            ResourceScope::new(ResourceKind::Tool, tool_name).expect("scope"),
            PolicyDecision::Allow {
                reason: "fixture allows tool".to_string(),
            },
        );
    }
    policy
}

fn tool_call_stream(
    call_id: &str,
    tool_name: &str,
    input_json: serde_json::Value,
) -> ModelStreamResult {
    vec![
        ModelStreamEvent::Started {
            request_id: format!("request-{call_id}"),
        },
        ModelStreamEvent::ToolCall {
            call_id: call_id.to_string(),
            tool_name: tool_name.to_string(),
            input_json,
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
            text: "Updated notes.txt and verified it.".to_string(),
        },
        ModelStreamEvent::Completed {
            finish_reason: "stop".to_string(),
            output_token_count: 34,
        },
    ]
}

fn local_coding_streams() -> Vec<ModelStreamResult> {
    vec![
        tool_call_stream(
            "call-read",
            "workspace.read_file",
            json!({ "path": "notes.txt" }),
        ),
        tool_call_stream(
            "call-edit",
            "workspace.edit_file",
            json!({
                "path": "notes.txt",
                "expected": "hello\n",
                "replacement": "hello from agent\n",
                "apply": true
            }),
        ),
        tool_call_stream(
            "call-shell",
            "workspace.shell_command",
            json!({
                "program": "git",
                "args": ["status", "--short"],
                "cwd": ".",
                "timeout_ms": 30_000
            }),
        ),
        tool_call_stream(
            "call-git-status",
            "workspace.git_status",
            json!({ "cwd": "." }),
        ),
        final_stream(),
    ]
}

struct ScriptedProvider {
    streams: Mutex<VecDeque<ModelStreamResult>>,
}

impl ScriptedProvider {
    fn new(streams: Vec<ModelStreamResult>) -> Arc<Self> {
        Arc::new(Self {
            streams: Mutex::new(streams.into()),
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
        RegisteredModelProvider::new(
            config,
            capabilities(),
            ScriptedProvider::new(local_coding_streams()),
        )
        .expect("registered provider"),
    ])
    .expect("registry")
}

#[tokio::test]
async fn comprehensive_turn_completes_local_coding_workflow() {
    let ids = runtime_ids();
    let config = provider_config();
    let fixture = local_coding_fixture();
    let mut effects = InMemoryEventStore::default();

    let outcome = ComprehensiveTurnRunner::run_with_idempotency(
        input(ids, config.clone()),
        &registry(config),
        ComprehensiveTurnResources {
            tools: &fixture.tools,
            policy: &fixture.policy,
        },
        &mut effects,
    )
    .await
    .expect("local coding turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(
        fs::read_to_string(fixture.workspace.path().join("notes.txt")).expect("edited file"),
        "hello from agent\n"
    );
    assert_eq!(
        outcome.final_message.as_deref(),
        Some("Updated notes.txt and verified it.")
    );

    let phases: Vec<&'static str> = outcome
        .events
        .iter()
        .filter_map(|event| match event.payload {
            RuntimeEventPayload::ToolCallProposed(_) => Some("proposed"),
            RuntimeEventPayload::PermissionDecisionRecorded(_) => Some("decision"),
            RuntimeEventPayload::ToolExecutionStarted(_) => Some("started"),
            RuntimeEventPayload::ToolExecutionCompleted(_) => Some("completed"),
            RuntimeEventPayload::ToolObservationRecorded(_) => Some("observed"),
            RuntimeEventPayload::FinalResponseProduced(_) => Some("final"),
            _ => None,
        })
        .collect();

    assert!(
        phases.windows(5).any(|window| {
            window == ["proposed", "decision", "started", "completed", "observed"]
        })
    );
    assert_eq!(phases.last(), Some(&"final"));

    let proposed_tools: Vec<&str> = outcome
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            RuntimeEventPayload::ToolCallProposed(call) => Some(call.tool_name.as_str()),
            _ => None,
        })
        .collect();
    assert!(proposed_tools.contains(&"workspace.read_file"));
    assert!(proposed_tools.contains(&"workspace.edit_file"));
    assert!(proposed_tools.contains(&"workspace.shell_command"));
    assert!(proposed_tools.contains(&"workspace.git_status"));

    let replayed_final = outcome
        .events
        .iter()
        .find_map(|event| match &event.payload {
            RuntimeEventPayload::FinalResponseProduced(response) => Some(response.message.as_str()),
            _ => None,
        });
    assert_eq!(replayed_final, Some("Updated notes.txt and verified it."));
}
