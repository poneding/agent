use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentLoopBudget, AgentLoopTerminalState, AgentRuntimeConfig,
    ComprehensiveTurnInput, ComprehensiveTurnResources, ComprehensiveTurnRunner, ConfigLayer,
    ConfigLayerKind, ConfigOverride, RuntimeIds, RuntimeModelConfig,
    build_effective_runtime_config,
};
use agent_mcp::{
    FixtureMcpClient, McpGateway, McpInitializeResult, McpServerCapabilities, McpServerConfig,
    McpServerId, McpToolCallResult, McpToolDescriptor, McpTransportConfig,
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
use agent_tool::ToolRegistry;
use agent_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};
use async_trait::async_trait;
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

fn input(config: ModelProviderConfig) -> ComprehensiveTurnInput {
    ComprehensiveTurnInput {
        ids: runtime_ids(),
        user_message: "use mcp".to_string(),
        effective_config: effective_config(config),
        requirements: requirements(),
        loop_budget: AgentLoopBudget {
            max_model_turns: 4,
            max_tool_calls: 4,
            max_wall_time_ms: 30_000,
            max_observation_bytes: 4096,
            max_consecutive_invalid_tool_calls: 2,
        },
    }
}

fn server_id() -> McpServerId {
    McpServerId::new("filesystem").unwrap()
}

fn mcp_tools(client: Arc<FixtureMcpClient>) -> ToolRegistry {
    let config = McpServerConfig::new(
        server_id(),
        "Filesystem",
        McpTransportConfig::stdio("mcp-filesystem"),
    )
    .unwrap();
    let gateway = McpGateway::new(client);
    let report = futures::executor::block_on(gateway.discover_tools(&[config]));
    let mut registry = ToolRegistry::default();
    gateway.register_tools(&mut registry, &report).unwrap();
    registry
}

fn mcp_client() -> Arc<FixtureMcpClient> {
    let server_id = server_id();
    Arc::new(
        FixtureMcpClient::new()
            .with_server(
                server_id.clone(),
                McpInitializeResult::new(
                    server_id.clone(),
                    "2025-11-25",
                    "fixture",
                    None::<String>,
                    McpServerCapabilities {
                        tools: true,
                        prompts: false,
                        resources: false,
                    },
                ),
            )
            .with_tools(
                server_id.clone(),
                vec![
                    McpToolDescriptor::new(
                        server_id.clone(),
                        "read_file",
                        Some("Read File"),
                        "Read from MCP",
                        json!({"type": "object"}),
                    )
                    .unwrap(),
                ],
            )
            .with_tool_result(
                server_id,
                "read_file",
                McpToolCallResult::new(json!({"content": "hello from mcp"})),
            ),
    )
}

fn policy(decision: PolicyDecision) -> StaticPolicyEngine {
    StaticPolicyEngine::default().with_decision(
        CapabilityAction::Read,
        ResourceScope::new(ResourceKind::Tool, "mcp.filesystem.read_file").expect("scope"),
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
            tool_name: "mcp.filesystem.read_file".to_string(),
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
            text: "mcp complete".to_string(),
        },
        ModelStreamEvent::Completed {
            finish_reason: "stop".to_string(),
            output_token_count: 2,
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

fn model_registry(
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
async fn mcp_gateway_tool_runs_through_comprehensive_turn_policy_and_events() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, requests) =
        RecordingProvider::scripted(vec![tool_call_stream(), final_stream()]);
    let models = model_registry(config, provider);
    let client = mcp_client();
    let tools = mcp_tools(client.clone());
    let policy = policy(PolicyDecision::Allow {
        reason: "allow mcp fixture".to_string(),
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
    assert_eq!(outcome.final_message.as_deref(), Some("mcp complete"));
    assert_eq!(client.call_count("filesystem", "read_file"), 1);
    assert!(
        requests.lock().unwrap()[1]
            .messages
            .iter()
            .any(|message| message.contains("hello from mcp"))
    );
    assert!(outcome.events.iter().any(|event| matches!(
        event.payload,
        RuntimeEventPayload::PermissionDecisionRecorded(_)
    )));
    assert!(outcome.events.iter().any(|event| matches!(
        event.payload,
        RuntimeEventPayload::ToolExecutionCompleted(_)
    )));
}

#[tokio::test]
async fn denied_mcp_gateway_tool_never_calls_remote_client() {
    let config = provider_config();
    let input = input(config.clone());
    let (provider, requests) =
        RecordingProvider::scripted(vec![tool_call_stream(), final_stream()]);
    let models = model_registry(config, provider);
    let client = mcp_client();
    let tools = mcp_tools(client.clone());
    let policy = policy(PolicyDecision::Deny {
        reason: "deny mcp fixture".to_string(),
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
    assert!(
        requests.lock().unwrap()[1]
            .messages
            .iter()
            .any(|message| message.contains("denied by policy"))
    );
    assert_eq!(client.call_count("filesystem", "read_file"), 0);
}
