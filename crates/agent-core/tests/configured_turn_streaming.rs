use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentRuntimeConfig, ConfigLayer, ConfigLayerKind, ConfigOverride,
    ConfiguredTurnInput, ConfiguredTurnRunner, RuntimeIds, RuntimeModelConfig,
    build_effective_runtime_config,
};
use agent_model::{
    ModelCapabilities, ModelEndpointConfig, ModelError, ModelEventStream, ModelFallbackChain,
    ModelInvocationPolicy, ModelProvider, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, ModelProviderRegistry, ModelRequest, ModelRequestRequirements,
    ModelRetryPolicy, ModelStreamEvent, RegisteredModelProvider,
};
use agent_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};
use async_trait::async_trait;
use futures_util::{FutureExt, Stream, StreamExt};

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

fn provider_config() -> ModelProviderConfig {
    ModelProviderConfig::new(
        "streaming",
        ModelProviderKind::Fake,
        "streaming-model",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid provider")
}

fn effective_config(provider: ModelProviderConfig) -> agent_core::EffectiveRuntimeConfig {
    build_effective_runtime_config(
        vec![ConfigLayer {
            kind: ConfigLayerKind::BuiltIn,
            source: "test".to_string(),
            config: AgentRuntimeConfig {
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
                terminal: Default::default(),
            },
        }],
        ConfigOverride::default(),
    )
    .expect("effective config")
}

fn configured_input(provider: ModelProviderConfig) -> ConfiguredTurnInput {
    ConfiguredTurnInput {
        ids: runtime_ids(),
        user_message: "stream this turn".to_string(),
        conversation_history: std::sync::Arc::new(Vec::new()),
        effective_config: std::sync::Arc::new(effective_config(provider)),
        requirements: requirements(),
    }
}

fn registry(
    provider: ModelProviderConfig,
    model_provider: Arc<dyn ModelProvider>,
) -> ModelProviderRegistry {
    ModelProviderRegistry::new(vec![
        RegisteredModelProvider::new(provider, capabilities(), model_provider)
            .expect("registered provider"),
    ])
    .expect("registry")
}

#[derive(Clone, Debug, Default)]
struct CompletionGate {
    released: Arc<AtomicBool>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl CompletionGate {
    fn release(&self) {
        self.released.store(true, Ordering::SeqCst);
        if let Some(waker) = self.waker.lock().expect("waker lock").take() {
            waker.wake();
        }
    }
}

struct GatedModelStream {
    gate: CompletionGate,
    step: u8,
}

impl Stream for GatedModelStream {
    type Item = Result<ModelStreamEvent, ModelError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.step {
            0 => {
                this.step = 1;
                Poll::Ready(Some(Ok(ModelStreamEvent::Started {
                    request_id: "gated-request".to_string(),
                })))
            }
            1 => {
                this.step = 2;
                Poll::Ready(Some(Ok(ModelStreamEvent::Delta {
                    text: "hel".to_string(),
                })))
            }
            2 if this.gate.released.load(Ordering::SeqCst) => {
                this.step = 3;
                Poll::Ready(Some(Ok(ModelStreamEvent::Completed {
                    finish_reason: "stop".to_string(),
                    output_token_count: 3,
                })))
            }
            2 => {
                *this.gate.waker.lock().expect("waker lock") = Some(cx.waker().clone());
                Poll::Pending
            }
            _ => Poll::Ready(None),
        }
    }
}

#[derive(Debug)]
struct GatedProvider {
    gate: CompletionGate,
}

#[async_trait]
impl ModelProvider for GatedProvider {
    async fn stream(&self, _request: ModelRequest) -> Result<ModelEventStream, ModelError> {
        Ok(Box::pin(GatedModelStream {
            gate: self.gate.clone(),
            step: 0,
        }))
    }
}

#[tokio::test]
async fn configured_turn_streaming_yields_deltas_before_final_response() {
    let provider = provider_config();
    let gate = CompletionGate::default();
    let registry = registry(
        provider.clone(),
        Arc::new(GatedProvider { gate: gate.clone() }),
    );

    let turn_stream = tokio::time::timeout(
        Duration::from_millis(100),
        ConfiguredTurnRunner::run_stream(configured_input(provider), &registry),
    )
    .await
    .expect("run_stream should return before provider completion is released")
    .expect("configured turn stream");

    assert_eq!(turn_stream.selected_provider, "streaming");
    assert_eq!(turn_stream.selected_model, "streaming-model");
    assert_eq!(turn_stream.attempts.len(), 1);

    let mut events = turn_stream.events;
    let mut saw_delta = false;
    while let Some(event) = events.next().await {
        let event = event.expect("event before completion gate");
        match event.payload {
            RuntimeEventPayload::ModelOutputDelta(delta) => {
                assert_eq!(delta.delta, "hel");
                saw_delta = true;
                break;
            }
            RuntimeEventPayload::FinalResponseProduced(_) => {
                panic!("final response should not be produced before first delta")
            }
            _ => {}
        }
    }

    assert!(saw_delta, "delta should be yielded before completion");
    assert!(
        events.next().now_or_never().is_none(),
        "stream should wait for provider completion after the first delta"
    );

    gate.release();

    let remaining = events
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("remaining events");
    assert!(remaining.iter().any(|event| {
        matches!(
            &event.payload,
            RuntimeEventPayload::FinalResponseProduced(response)
                if response.message == "hel"
        )
    }));
}
