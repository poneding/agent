use std::cell::Cell;
use std::collections::VecDeque;
use std::fs;
use std::sync::{Arc, Mutex};

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentLoopBudget, AgentLoopTerminalState, AgentRuntimeConfig,
    ApprovalCoordinator, ComprehensiveApprovalContinuation, ComprehensiveTurnInput,
    ComprehensiveTurnResources, ComprehensiveTurnRunner, ComprehensiveTurnStreamOptions,
    ConfigLayer, ConfigLayerKind, ConfigOverride, RuntimeIds, RuntimeModelConfig,
    build_effective_runtime_config,
};
use agent_model::{
    ModelCapabilities, ModelEndpointConfig, ModelError, ModelEventStream, ModelFallbackChain,
    ModelInvocationPolicy, ModelMessage, ModelProvider, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, ModelProviderRegistry, ModelRequest, ModelRequestRequirements,
    ModelRetryPolicy, ModelStreamEvent, ModelStreamResult, RegisteredModelProvider,
    model_events_to_stream,
};
use agent_policy::{
    ApprovalResponse, CapabilityAction, PolicyDecision, ResourceKind, ResourceScope,
    StaticPolicyEngine,
};
use agent_sandbox::WorkspaceSandbox;
use agent_storage::InMemoryEventStore;
use agent_tool::ToolRegistry;
use agent_tool::builtins::ReadFileTool;
use agent_types::{
    ConversationId, CorrelationId, ReasoningEffort, RuntimeEventPayload, Timestamp, TurnId,
    WorkspaceId,
};
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

fn input(config: ModelProviderConfig) -> ComprehensiveTurnInput {
    ComprehensiveTurnInput {
        ids: runtime_ids(),
        user_message: "read README and summarize".to_string(),
        conversation_history: std::sync::Arc::new(Vec::new()),
        effective_config: std::sync::Arc::new(effective_config(config)),
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
    tool_call_stream_with_id("call-1")
}

fn tool_call_stream_with_id(call_id: &str) -> ModelStreamResult {
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

fn multi_tool_call_stream() -> ModelStreamResult {
    vec![
        ModelStreamEvent::Started {
            request_id: "request-multi".to_string(),
        },
        ModelStreamEvent::ToolCall {
            call_id: "call-1".to_string(),
            tool_name: "workspace.read_file".to_string(),
            input_json: json!({ "path": "README.md" }),
        },
        ModelStreamEvent::ToolCall {
            call_id: "call-2".to_string(),
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
    assert!(
        requests[0]
            .tools
            .iter()
            .any(|tool| tool.name == "workspace.read_file")
    );
    assert!(requests[1].messages.iter().any(|message| matches!(
        message,
        ModelMessage::ToolResult { content, .. }
            if content.contains("tool observation") && content.contains("hello from tool")
    )));
    assert!(matches!(
        requests[1].messages.last(),
        Some(ModelMessage::ToolResult { call_id, .. }) if call_id == "call-1"
    ));
    assert_eq!(
        requests[1]
            .messages
            .iter()
            .filter(|message| matches!(
                message,
                ModelMessage::User { content }
                    if content == "read README and summarize"
            ))
            .count(),
        1,
        "the original user message is present once and is not re-appended after tool results"
    );
}

#[tokio::test]
async fn comprehensive_turn_keeps_multi_tool_continuations_after_the_user_message() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, requests) = RecordingProvider::scripted(vec![
        tool_call_stream_with_id("call-1"),
        tool_call_stream_with_id("call-2"),
        final_stream(),
    ]);
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
    assert_eq!(
        outcome
            .events
            .iter()
            .filter(|event| matches!(&event.payload, RuntimeEventPayload::UserInputReceived(_)))
            .count(),
        1,
        "one top-level user input is persisted regardless of internal model turns"
    );

    let requests = requests.lock().expect("requests");
    assert_eq!(requests.len(), 3);
    assert_eq!(
        requests[2]
            .messages
            .iter()
            .filter(|message| matches!(
                message,
                ModelMessage::User { content }
                    if content == "read README and summarize"
            ))
            .count(),
        1
    );
    assert!(matches!(
        requests[2].messages.as_slice(),
        [
            ModelMessage::User { .. },
            ModelMessage::AssistantToolCall { call_id: first, .. },
            ModelMessage::ToolResult { call_id: first_result, .. },
            ModelMessage::AssistantToolCall { call_id: second, .. },
            ModelMessage::ToolResult { call_id: second_result, .. },
        ] if first == "call-1"
            && first_result == "call-1"
            && second == "call-2"
            && second_result == "call-2"
    ));
}

#[tokio::test]
async fn comprehensive_turn_executes_all_tool_calls_from_one_model_response() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, requests) =
        RecordingProvider::scripted(vec![multi_tool_call_stream(), final_stream()]);
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
    let proposed_count = outcome
        .events
        .iter()
        .filter(|event| matches!(event.payload, RuntimeEventPayload::ToolCallProposed(_)))
        .count();
    assert!((2..=4).contains(&proposed_count));

    let requests = requests.lock().expect("requests");
    assert_eq!(requests.len(), 2);
    assert!(matches!(
        requests[1].messages.as_slice(),
        [
            ModelMessage::User { .. },
            ModelMessage::AssistantToolCall { call_id: first, .. },
            ModelMessage::ToolResult { call_id: first_result, .. },
            ModelMessage::AssistantToolCall { call_id: second, .. },
            ModelMessage::ToolResult { call_id: second_result, .. },
        ] if first == "call-1"
            && first_result == "call-1"
            && second == "call-2"
            && second_result == "call-2"
    ));
    assert_eq!(
        requests[1]
            .messages
            .iter()
            .filter(|message| matches!(
                message,
                ModelMessage::User { content }
                    if content == "read README and summarize"
            ))
            .count(),
        1
    );
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
    assert!(outcome.pending_approval.is_some());
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

#[tokio::test]
async fn comprehensive_turn_registers_pending_approval_before_streaming_request() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, _requests) =
        RecordingProvider::scripted(vec![tool_call_stream(), final_stream()]);
    let models = registry(config, provider);
    let (_temp, tools) = read_file_registry();
    let policy = policy(PolicyDecision::Ask {
        reason: "needs approval".to_string(),
    });
    let approval_registered = Cell::new(false);
    let approval_request_streamed_after_registration = Cell::new(false);

    let outcome = ComprehensiveTurnRunner::run_with_resources_streaming_and_approval_sink(
        input,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
        ComprehensiveTurnStreamOptions::default(),
        |event| {
            if matches!(event.payload, RuntimeEventPayload::ApprovalRequested(_)) {
                approval_request_streamed_after_registration.set(approval_registered.get());
            }
            Ok(())
        },
        |pending| {
            assert_eq!(pending.suspended.call.tool_name, "workspace.read_file");
            approval_registered.set(true);
            Ok(())
        },
    )
    .await
    .expect("turn");

    assert_eq!(
        outcome.terminal_state,
        AgentLoopTerminalState::WaitingApproval
    );
    assert!(approval_registered.get());
    assert!(approval_request_streamed_after_registration.get());
}

#[tokio::test]
async fn comprehensive_turn_continues_after_approved_tool_execution() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, requests) =
        RecordingProvider::scripted(vec![tool_call_stream(), final_stream()]);
    let models = registry(config, provider);
    let (_temp, tools) = read_file_registry();
    let policy = policy(PolicyDecision::Ask {
        reason: "needs approval".to_string(),
    });
    let mut coordinator = ApprovalCoordinator::default();
    let mut continuation: Option<ComprehensiveApprovalContinuation> = None;

    let suspended = ComprehensiveTurnRunner::run_with_resources_streaming_and_approval_sink(
        input,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
        ComprehensiveTurnStreamOptions {
            coding_context: false,
            cancellation: Default::default(),
            reasoning_effort: Some(ReasoningEffort::High),
        },
        |_event| Ok(()),
        |pending| {
            coordinator
                .register_suspended_tool(
                    pending.suspended.ids,
                    pending.suspended.model_turn_index,
                    pending.suspended.call,
                    pending.suspended.approval,
                )
                .expect("register suspended approval");
            continuation = Some(pending.continuation);
            Ok(())
        },
    )
    .await
    .expect("suspended turn");

    let approval = suspended
        .pending_approval
        .as_ref()
        .expect("pending approval");
    let mut effects = InMemoryEventStore::default();
    let approval_outcome = coordinator
        .resume_tool_approval(
            ApprovalResponse::allow(
                approval.approval.id,
                "approved in test",
                None,
                Timestamp::now_utc(),
            )
            .expect("approval response"),
            &tools,
            &mut effects,
        )
        .expect("resume approval");

    let continued = ComprehensiveTurnRunner::continue_after_approval_streaming(
        continuation.expect("continuation"),
        approval_outcome,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
        ComprehensiveTurnStreamOptions::default(),
        |_event| Ok(()),
        |_pending| Ok(()),
    )
    .await
    .expect("continued turn");

    assert_eq!(continued.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(continued.final_message.as_deref(), Some("read complete"));
    let requests = requests.lock().expect("requests");
    assert_eq!(requests.len(), 2);
    assert!(
        requests
            .iter()
            .all(|request| request.reasoning_effort == Some(ReasoningEffort::High))
    );
    assert!(requests[1].messages.iter().any(|message| matches!(
        message,
        ModelMessage::ToolResult { content, .. }
            if content.contains("tool observation") && content.contains("hello from tool")
    )));
}

#[tokio::test]
async fn comprehensive_turn_streams_model_and_tool_events_to_sink() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, _requests) =
        RecordingProvider::scripted(vec![tool_call_stream(), final_stream()]);
    let models = registry(config, provider);
    let (_temp, tools) = read_file_registry();
    let policy = policy(PolicyDecision::Allow {
        reason: "read-only fixture".to_string(),
    });
    let mut streamed_payloads = Vec::new();

    let outcome = ComprehensiveTurnRunner::run_with_resources_streaming(
        input,
        &models,
        ComprehensiveTurnResources {
            tools: &tools,
            policy: &policy,
        },
        ComprehensiveTurnStreamOptions::default(),
        |event| {
            streamed_payloads.push(event.payload);
            Ok(())
        },
    )
    .await
    .expect("turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(streamed_payloads.len(), outcome.events.len());
    assert!(
        streamed_payloads
            .iter()
            .any(|payload| { matches!(payload, RuntimeEventPayload::ToolCallProposed(_)) })
    );
    assert!(
        streamed_payloads
            .iter()
            .any(|payload| { matches!(payload, RuntimeEventPayload::ToolExecutionStarted(_)) })
    );
    assert!(
        streamed_payloads
            .iter()
            .any(|payload| { matches!(payload, RuntimeEventPayload::ToolObservationRecorded(_)) })
    );
    assert!(
        streamed_payloads
            .iter()
            .any(|payload| { matches!(payload, RuntimeEventPayload::FinalResponseProduced(_)) })
    );
}
