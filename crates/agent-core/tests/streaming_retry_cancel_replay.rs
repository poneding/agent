use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentRuntimeConfig, CancellationToken, ConfigLayer,
    ConfigLayerKind, ConfigOverride, ConfiguredTurnError, ConfiguredTurnInput,
    ConfiguredTurnRunner, ConfiguredTurnStreamOptions, RuntimeIds, RuntimeModelConfig,
    build_effective_runtime_config,
};
use agent_model::{
    ModelCapabilities, ModelEndpointConfig, ModelError, ModelEventStream, ModelFailureKind,
    ModelFallbackChain, ModelInvocationPolicy, ModelProvider, ModelProviderCatalog,
    ModelProviderConfig, ModelProviderKind, ModelProviderRegistry, ModelRequest,
    ModelRequestRequirements, ModelRetryPolicy, ModelStreamEvent, RegisteredModelProvider,
};
use agent_storage::{EventStore, InMemoryEventStore, ReplayCursor};
use agent_types::{
    ConversationId, CorrelationId, RuntimeEventPayload, TurnId, TurnState, WorkspaceId,
};
use async_trait::async_trait;
use futures_util::{Stream, StreamExt, stream};

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
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

fn provider_config(id: &str, model: &str) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Fake,
        model,
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid provider")
}

fn invocation_policy(
    max_attempts: u32,
    retryable_failures: Vec<ModelFailureKind>,
    fallback_provider_ids: Vec<&str>,
) -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(max_attempts, retryable_failures).expect("valid retry"),
        ModelFallbackChain::new(fallback_provider_ids).expect("valid fallback"),
    )
    .expect("valid policy")
}

fn effective_config(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
    invocation_policy: ModelInvocationPolicy,
) -> agent_core::EffectiveRuntimeConfig {
    build_effective_runtime_config(
        vec![ConfigLayer {
            kind: ConfigLayerKind::BuiltIn,
            source: "test".to_string(),
            config: AgentRuntimeConfig {
                schema_version: AGENT_CONFIG_SCHEMA_VERSION,
                model: RuntimeModelConfig {
                    default_provider: default_provider.to_string(),
                    catalog: ModelProviderCatalog::new(providers).expect("valid catalog"),
                    invocation_policy,
                },
            },
        }],
        ConfigOverride::default(),
    )
    .expect("effective config")
}

fn configured_input(
    ids: RuntimeIds,
    effective_config: agent_core::EffectiveRuntimeConfig,
) -> ConfiguredTurnInput {
    ConfiguredTurnInput {
        ids,
        user_message: "stream with policy".to_string(),
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

fn provider_error(code: &str, retryable: bool) -> ModelError {
    ModelError::Provider {
        code: code.to_string(),
        message: "redacted provider failure".to_string(),
        retryable,
    }
}

fn stream_text_chunks(chunks: &[&str]) -> Vec<Result<ModelStreamEvent, ModelError>> {
    let mut events = vec![Ok(ModelStreamEvent::Started {
        request_id: "request-ok".to_string(),
    })];
    events.extend(chunks.iter().map(|chunk| {
        Ok(ModelStreamEvent::Delta {
            text: (*chunk).to_string(),
        })
    }));
    events.push(Ok(ModelStreamEvent::Completed {
        finish_reason: "stop".to_string(),
        output_token_count: chunks.iter().map(|chunk| chunk.len() as u32).sum(),
    }));
    events
}

fn stream_error_before_delta(
    code: &str,
    retryable: bool,
) -> Vec<Result<ModelStreamEvent, ModelError>> {
    vec![
        Ok(ModelStreamEvent::Started {
            request_id: format!("request-{code}"),
        }),
        Err(provider_error(code, retryable)),
    ]
}

enum ProviderOutcome {
    SetupError(ModelError),
    Stream(Vec<Result<ModelStreamEvent, ModelError>>),
    PendingAfterDelta,
}

struct ScriptedProvider {
    outcomes: Mutex<VecDeque<ProviderOutcome>>,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}

impl ScriptedProvider {
    fn new(outcomes: Vec<ProviderOutcome>) -> (Arc<Self>, Arc<Mutex<Vec<ModelRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let provider = Arc::new(Self {
            outcomes: Mutex::new(outcomes.into()),
            requests: Arc::clone(&requests),
        });
        (provider, requests)
    }
}

#[async_trait]
impl ModelProvider for ScriptedProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        self.requests.lock().expect("requests lock").push(request);
        match self
            .outcomes
            .lock()
            .expect("outcomes lock")
            .pop_front()
            .unwrap_or_else(|| {
                ProviderOutcome::SetupError(provider_error("script.exhausted", false))
            }) {
            ProviderOutcome::SetupError(error) => Err(error),
            ProviderOutcome::Stream(events) => Ok(Box::pin(stream::iter(events))),
            ProviderOutcome::PendingAfterDelta => Ok(Box::pin(PendingAfterDeltaStream { step: 0 })),
        }
    }
}

struct PendingAfterDeltaStream {
    step: u8,
}

impl Stream for PendingAfterDeltaStream {
    type Item = Result<ModelStreamEvent, ModelError>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.step {
            0 => {
                this.step = 1;
                Poll::Ready(Some(Ok(ModelStreamEvent::Started {
                    request_id: "request-pending".to_string(),
                })))
            }
            1 => {
                this.step = 2;
                Poll::Ready(Some(Ok(ModelStreamEvent::Delta {
                    text: "partial".to_string(),
                })))
            }
            _ => Poll::Pending,
        }
    }
}

#[tokio::test]
async fn streaming_retry_retries_retryable_error_before_delta() {
    let primary_config = provider_config("primary", "primary-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone()],
        invocation_policy(2, vec![ModelFailureKind::Timeout], Vec::new()),
    );
    let (primary, requests) = ScriptedProvider::new(vec![
        ProviderOutcome::Stream(stream_error_before_delta("model.timeout", true)),
        ProviderOutcome::Stream(stream_text_chunks(&["ready"])),
    ]);
    let registry =
        ModelProviderRegistry::new(vec![registered(primary_config, primary)]).expect("registry");

    let outcome = ConfiguredTurnRunner::run(configured_input(runtime_ids(), effective), &registry)
        .await
        .expect("retryable stream failure recovers");

    assert_eq!(outcome.final_message, "ready");
    assert_eq!(outcome.attempts.len(), 2);
    assert_eq!(requests.lock().expect("requests").len(), 2);
}

#[tokio::test]
async fn streaming_retry_does_not_retry_auth_error() {
    let primary_config = provider_config("primary", "primary-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone()],
        invocation_policy(3, vec![ModelFailureKind::InvalidCredentials], Vec::new()),
    );
    let (primary, requests) = ScriptedProvider::new(vec![ProviderOutcome::Stream(
        stream_error_before_delta("model.invalid_credentials", false),
    )]);
    let registry =
        ModelProviderRegistry::new(vec![registered(primary_config, primary)]).expect("registry");

    let error = ConfiguredTurnRunner::run(configured_input(runtime_ids(), effective), &registry)
        .await
        .expect_err("non-retryable auth error stops");

    assert!(matches!(
        error,
        ConfiguredTurnError::Model { attempts, .. } if attempts.len() == 1
    ));
    assert_eq!(requests.lock().expect("requests").len(), 1);
}

#[tokio::test]
async fn streaming_retry_uses_fallback_provider_after_stream_failure() {
    let primary_config = provider_config("primary", "primary-model");
    let backup_config = provider_config("backup", "backup-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone(), backup_config.clone()],
        invocation_policy(1, vec![ModelFailureKind::Transport], vec!["backup"]),
    );
    let (primary, primary_requests) = ScriptedProvider::new(vec![ProviderOutcome::Stream(
        stream_error_before_delta("model.transport", true),
    )]);
    let (backup, backup_requests) =
        ScriptedProvider::new(vec![ProviderOutcome::Stream(stream_text_chunks(&[
            "fallback",
        ]))]);
    let registry = ModelProviderRegistry::new(vec![
        registered(primary_config, primary),
        registered(backup_config, backup),
    ])
    .expect("registry");

    let outcome = ConfiguredTurnRunner::run(configured_input(runtime_ids(), effective), &registry)
        .await
        .expect("fallback stream succeeds");

    assert_eq!(outcome.final_message, "fallback");
    assert_eq!(outcome.attempts.len(), 2);
    assert_eq!(outcome.attempts[0].provider_id, "primary");
    assert_eq!(outcome.attempts[1].provider_id, "backup");
    assert_eq!(primary_requests.lock().expect("requests").len(), 1);
    assert_eq!(backup_requests.lock().expect("requests").len(), 1);
}

#[tokio::test]
async fn streaming_cancellation_emits_cancelled_state() {
    let primary_config = provider_config("primary", "primary-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone()],
        invocation_policy(1, Vec::new(), Vec::new()),
    );
    let (primary, _requests) = ScriptedProvider::new(vec![ProviderOutcome::PendingAfterDelta]);
    let registry =
        ModelProviderRegistry::new(vec![registered(primary_config, primary)]).expect("registry");
    let cancellation = CancellationToken::new();
    let turn_stream = ConfiguredTurnRunner::run_stream_with_options(
        configured_input(runtime_ids(), effective),
        &registry,
        ConfiguredTurnStreamOptions {
            cancellation: cancellation.clone(),
        },
    )
    .await
    .expect("turn stream");

    let mut events = turn_stream.events;
    while let Some(event) = events.next().await {
        let event = event.expect("event before cancellation");
        if matches!(event.payload, RuntimeEventPayload::ModelOutputDelta(_)) {
            break;
        }
    }

    cancellation.cancel();
    let remaining = tokio::time::timeout(Duration::from_millis(100), events.collect::<Vec<_>>())
        .await
        .expect("cancelled stream should finish")
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("remaining events");

    assert!(remaining.iter().any(|event| {
        matches!(
            &event.payload,
            RuntimeEventPayload::TurnStateChanged(change)
                if change.to == TurnState::Cancelled
        )
    }));
    assert!(
        !remaining.iter().any(|event| {
            matches!(event.payload, RuntimeEventPayload::FinalResponseProduced(_))
        })
    );
}

#[tokio::test]
async fn streaming_replay_reconstructs_final_response_from_events() {
    let ids = runtime_ids();
    let primary_config = provider_config("primary", "primary-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone()],
        invocation_policy(1, Vec::new(), Vec::new()),
    );
    let (primary, _requests) =
        ScriptedProvider::new(vec![ProviderOutcome::Stream(stream_text_chunks(&[
            "hel", "lo",
        ]))]);
    let registry =
        ModelProviderRegistry::new(vec![registered(primary_config, primary)]).expect("registry");
    let outcome = ConfiguredTurnRunner::run(configured_input(ids, effective), &registry)
        .await
        .expect("streaming turn");
    let mut store = InMemoryEventStore::default();
    for event in outcome.events {
        store.append(event).expect("append event");
    }

    let replayed = ReplayCursor::new(ids.conversation_id)
        .collect(&store)
        .expect("replay conversation");
    let delta_text = replayed
        .iter()
        .filter_map(|event| match &event.event.payload {
            RuntimeEventPayload::ModelOutputDelta(delta) => Some(delta.delta.as_str()),
            _ => None,
        })
        .collect::<String>();
    let final_response = replayed
        .iter()
        .filter_map(|event| match &event.event.payload {
            RuntimeEventPayload::FinalResponseProduced(response) => Some(response.message.as_str()),
            _ => None,
        })
        .next_back();

    assert_eq!(delta_text, "hello");
    assert_eq!(final_response, Some("hello"));
}
