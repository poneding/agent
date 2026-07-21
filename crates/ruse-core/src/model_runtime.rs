use std::collections::VecDeque;
use std::fmt;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ruse_context::{
    PromptAssemblyRequest, PromptBudget, PromptLayer, PromptLayerKind, PromptLayerPolicy,
    PromptLayerSensitivity, PromptLayerSource, PromptLayerVisibility, PromptMessage, PromptProfile,
    PromptProfileId, PromptRuntime, PromptSnapshot,
};
use ruse_model::{
    ModelCapabilities, ModelCapability, ModelError, ModelFailureKind, ModelInvocationPolicy,
    ModelMessage, ModelProvider, ModelProviderRegistry, ModelRegistryError, ModelRequest,
    ModelRequestRequirements, ModelStreamEvent, ModelStreamResult, ModelToolDefinition,
    RegisteredModelProvider,
};
use ruse_policy::PolicyEngine;
use ruse_storage::{EventStore, IdempotencyKey, IdempotencyStore};
use ruse_tool::ToolRegistry;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ApprovalResumeOutcome;
use crate::approval::SuspendedToolApproval;
use crate::config::EffectiveRuntimeConfig;
use crate::idempotency::IdempotentToolExecutor;
use crate::runtime::RuntimeIds;
use crate::tool_loop::{
    AccumulatedToolCall, ToolCallAccumulator, ToolCallRequest, ToolLoop, ToolLoopStatus,
    ToolObservationMessage,
};
use crate::turn::{CancellationToken, TurnEngine, TurnError, TurnInput, TurnStreamOptions};
use futures_util::{Stream, StreamExt, stream};
use ruse_types::{
    ActorRef, PromptLayerHashRecord, PromptSnapshotRecorded, ReasoningEffort, RuntimeError,
    RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp, TurnState,
};

#[derive(Clone)]
pub struct ConfiguredModelRuntime {
    pub provider_id: String,
    pub model: String,
    pub provider: Arc<dyn ModelProvider>,
    pub capabilities: ModelCapabilities,
    pub invocation_policy: ModelInvocationPolicy,
}

impl fmt::Debug for ConfiguredModelRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfiguredModelRuntime")
            .field("provider_id", &self.provider_id)
            .field("model", &self.model)
            .field("capabilities", &self.capabilities)
            .field("invocation_policy", &self.invocation_policy)
            .finish_non_exhaustive()
    }
}

pub struct ModelRuntimeResolver;

impl ModelRuntimeResolver {
    pub fn resolve(
        effective: &EffectiveRuntimeConfig,
        registry: &ModelProviderRegistry,
        requirements: &ModelRequestRequirements,
    ) -> Result<ConfiguredModelRuntime, ModelRuntimeError> {
        let selected_provider = effective.selected_provider.trim();
        if selected_provider.is_empty() {
            return Err(ModelRuntimeError::InvalidSelection {
                field: "model.selected_provider".to_string(),
                message: "selected provider must not be empty".to_string(),
            });
        }

        let selected_model = effective.selected_model.trim();
        if selected_model.is_empty() {
            return Err(ModelRuntimeError::InvalidSelection {
                field: "model.selected_model".to_string(),
                message: "selected model must not be empty".to_string(),
            });
        }

        let provider_config = effective
            .config
            .model
            .catalog
            .providers
            .iter()
            .find(|provider| provider.id == selected_provider)
            .ok_or_else(|| ModelRuntimeError::InvalidSelection {
                field: "model.selected_provider".to_string(),
                message: format!(
                    "selected provider {selected_provider} was not found in the effective catalog"
                ),
            })?;

        if !provider_config.enabled {
            return Err(ModelRuntimeError::ProviderDisabled {
                provider_id: selected_provider.to_string(),
            });
        }

        let registered = registry
            .get(selected_provider)
            .map_err(ModelRuntimeError::from_registry)?;
        let selected_missing = registered.capabilities.missing_for(requirements);
        if !selected_missing.is_empty() {
            if let Some(fallback) = first_compatible_fallback_runtime(
                effective,
                registry,
                requirements,
                selected_provider,
                selected_model,
                registered,
            )? {
                return Ok(fallback);
            }
            return Err(ModelRuntimeError::MissingCapabilities {
                provider_id: selected_provider.to_string(),
                missing: selected_missing,
            });
        }

        Ok(ConfiguredModelRuntime {
            provider_id: selected_provider.to_string(),
            model: selected_model.to_string(),
            provider: registered.provider(),
            capabilities: registered.capabilities.clone(),
            invocation_policy: effective.config.model.invocation_policy.clone(),
        })
    }
}

fn first_compatible_fallback_runtime(
    effective: &EffectiveRuntimeConfig,
    registry: &ModelProviderRegistry,
    requirements: &ModelRequestRequirements,
    selected_provider: &str,
    selected_model: &str,
    selected_registered: &RegisteredModelProvider,
) -> Result<Option<ConfiguredModelRuntime>, ModelRuntimeError> {
    let use_model_override = selected_model != selected_registered.config.default_model;
    for fallback_provider_id in &effective
        .config
        .model
        .invocation_policy
        .fallback
        .provider_ids
    {
        if fallback_provider_id == selected_provider {
            continue;
        }
        let Some(fallback_config) = effective
            .config
            .model
            .catalog
            .providers
            .iter()
            .find(|provider| provider.id == *fallback_provider_id)
        else {
            return Err(ModelRuntimeError::ProviderNotRegistered {
                provider_id: fallback_provider_id.clone(),
            });
        };
        if !fallback_config.enabled {
            continue;
        }
        let registered = match registry.require_capabilities(fallback_provider_id, requirements) {
            Ok(registered) => registered,
            Err(ModelRegistryError::MissingCapabilities { .. }) => continue,
            Err(error) => return Err(ModelRuntimeError::from_registry(error)),
        };
        let model = if use_model_override {
            selected_model.to_string()
        } else {
            registered.config.default_model.clone()
        };
        return Ok(Some(ConfiguredModelRuntime {
            provider_id: fallback_provider_id.clone(),
            model,
            provider: registered.provider(),
            capabilities: registered.capabilities.clone(),
            invocation_policy: effective.config.model.invocation_policy.clone(),
        }));
    }
    Ok(None)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelInvocationAttempt {
    pub provider_id: String,
    pub model: String,
    pub attempt_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelInvocationOutcome {
    pub provider_id: String,
    pub model: String,
    pub attempts: Vec<ModelInvocationAttempt>,
    pub stream: ModelStreamResult,
}

pub struct ModelInvocationRunner;

impl ModelInvocationRunner {
    pub async fn run(
        runtime: &ConfiguredModelRuntime,
        registry: &ModelProviderRegistry,
        requirements: &ModelRequestRequirements,
        request_template: ModelRequest,
    ) -> Result<ModelInvocationOutcome, ModelInvocationError> {
        let selected_provider = registry
            .get(&runtime.provider_id)
            .map_err(ModelInvocationError::selected_provider_unavailable)?;
        let use_model_override = runtime.model != selected_provider.config.default_model;
        let mut attempts = Vec::new();
        let mut next_attempt_index = 1;
        let mut last_failure = None;

        for _ in 0..runtime.invocation_policy.retry.max_attempts {
            let mut attempt = attempt(&runtime.provider_id, &runtime.model, next_attempt_index);
            next_attempt_index += 1;
            let started_at = Instant::now();

            let (result, request_id) = invoke(
                Arc::clone(&runtime.provider),
                &runtime.provider_id,
                &runtime.model,
                &request_template,
            )
            .await;
            match result {
                Ok(stream) => {
                    complete_attempt(&mut attempt, started_at.elapsed(), request_id);
                    attempts.push(attempt);
                    return Ok(ModelInvocationOutcome {
                        provider_id: runtime.provider_id.clone(),
                        model: runtime.model.clone(),
                        attempts,
                        stream,
                    });
                }
                Err(error) => {
                    complete_attempt(&mut attempt, started_at.elapsed(), request_id);
                    attempts.push(attempt);
                    let failure = InvocationFailure::new(
                        runtime.provider_id.clone(),
                        runtime.model.clone(),
                        error,
                    );
                    let should_retry = failure.retryable
                        && runtime
                            .invocation_policy
                            .retry
                            .is_retryable(failure.failure_kind)
                        && attempts.len() < runtime.invocation_policy.retry.max_attempts as usize;
                    last_failure = Some(failure);
                    if !should_retry {
                        break;
                    }
                }
            }
        }

        let mut skipped_fallback_missing = None;
        for fallback_provider_id in &runtime.invocation_policy.fallback.provider_ids {
            let registered = match registry.require_capabilities(fallback_provider_id, requirements)
            {
                Ok(registered) => registered,
                Err(ModelRegistryError::ProviderNotFound { id }) => {
                    return Err(ModelInvocationError::FallbackProviderNotRegistered {
                        provider_id: id,
                        attempts,
                    });
                }
                Err(ModelRegistryError::MissingCapabilities {
                    provider_id,
                    missing,
                }) => {
                    skipped_fallback_missing.get_or_insert((provider_id, missing));
                    continue;
                }
                Err(error) => {
                    return Err(ModelInvocationError::SelectedProviderUnavailable {
                        provider_id: fallback_provider_id.clone(),
                        message: error.to_string(),
                        attempts,
                    });
                }
            };
            let fallback_model = if use_model_override {
                runtime.model.clone()
            } else {
                registered.config.default_model.clone()
            };
            let mut attempt = attempt(fallback_provider_id, &fallback_model, next_attempt_index);
            next_attempt_index += 1;
            let started_at = Instant::now();

            let (result, request_id) = invoke(
                registered.provider(),
                fallback_provider_id,
                &fallback_model,
                &request_template,
            )
            .await;
            match result {
                Ok(stream) => {
                    complete_attempt(&mut attempt, started_at.elapsed(), request_id);
                    attempts.push(attempt);
                    return Ok(ModelInvocationOutcome {
                        provider_id: fallback_provider_id.clone(),
                        model: fallback_model,
                        attempts,
                        stream,
                    });
                }
                Err(error) => {
                    complete_attempt(&mut attempt, started_at.elapsed(), request_id);
                    attempts.push(attempt);
                    last_failure = Some(InvocationFailure::new(
                        fallback_provider_id.clone(),
                        fallback_model,
                        error,
                    ));
                }
            }
        }

        if let Some((provider_id, missing)) = skipped_fallback_missing {
            return Err(ModelInvocationError::FallbackMissingCapabilities {
                provider_id,
                missing,
                attempts,
            });
        }

        let failure = last_failure.expect("at least one selected provider attempt was recorded");
        Err(ModelInvocationError::InvocationFailed {
            provider_id: failure.provider_id,
            model: failure.model,
            failure_kind: failure.failure_kind,
            retryable: failure.retryable,
            message: failure.message,
            attempts,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfiguredTurnInput {
    pub ids: RuntimeIds,
    pub user_message: String,
    /// Shared across multi-step agent-loop model turns to avoid re-cloning
    /// history on every model call.
    pub conversation_history: Arc<Vec<PromptMessage>>,
    /// Shared config handle; provider resolve only needs a reference.
    pub effective_config: Arc<EffectiveRuntimeConfig>,
    pub requirements: ModelRequestRequirements,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComprehensiveTurnInput {
    pub ids: RuntimeIds,
    pub user_message: String,
    pub conversation_history: Arc<Vec<PromptMessage>>,
    pub effective_config: Arc<EffectiveRuntimeConfig>,
    pub requirements: ModelRequestRequirements,
    pub loop_budget: AgentLoopBudget,
}

impl ComprehensiveTurnInput {
    /// Build a comprehensive turn input, wrapping bulky fields in `Arc` so the
    /// agent loop can re-enter model turns without deep-cloning history/config.
    #[must_use]
    pub fn new(
        ids: RuntimeIds,
        user_message: impl Into<String>,
        conversation_history: Vec<PromptMessage>,
        effective_config: EffectiveRuntimeConfig,
        requirements: ModelRequestRequirements,
        loop_budget: AgentLoopBudget,
    ) -> Self {
        Self {
            ids,
            user_message: user_message.into(),
            conversation_history: Arc::new(conversation_history),
            effective_config: Arc::new(effective_config),
            requirements,
            loop_budget,
        }
    }
}

impl ConfiguredTurnInput {
    #[must_use]
    pub fn new(
        ids: RuntimeIds,
        user_message: impl Into<String>,
        conversation_history: Vec<PromptMessage>,
        effective_config: EffectiveRuntimeConfig,
        requirements: ModelRequestRequirements,
    ) -> Self {
        Self {
            ids,
            user_message: user_message.into(),
            conversation_history: Arc::new(conversation_history),
            effective_config: Arc::new(effective_config),
            requirements,
        }
    }

    #[must_use]
    pub fn from_shared(
        ids: RuntimeIds,
        user_message: impl Into<String>,
        conversation_history: Arc<Vec<PromptMessage>>,
        effective_config: Arc<EffectiveRuntimeConfig>,
        requirements: ModelRequestRequirements,
    ) -> Self {
        Self {
            ids,
            user_message: user_message.into(),
            conversation_history,
            effective_config,
            requirements,
        }
    }
}

pub struct ComprehensiveTurnResources<'a, P>
where
    P: PolicyEngine + ?Sized,
{
    pub tools: &'a ToolRegistry,
    pub policy: &'a P,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentLoopBudget {
    /// Number of model calls between progress checkpoints. Reaching the interval
    /// emits a warning; hard wall-time, tool-call, observation, and invalid-call
    /// budgets decide when a loop must stop.
    pub model_turn_checkpoint_interval: u32,
    pub max_tool_calls: u32,
    pub max_wall_time_ms: u64,
    pub max_observation_bytes: u64,
    pub max_consecutive_invalid_tool_calls: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentLoopTerminalState {
    Done,
    WaitingApproval,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentLoopFailureReason {
    MaxToolCallsExceeded,
    MaxWallTimeExceeded,
    MaxObservationBytesExceeded,
    MaxConsecutiveInvalidToolCallsExceeded,
    InvalidBudget,
    Runtime(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentLoopOutcome {
    pub terminal_state: AgentLoopTerminalState,
    pub final_message: Option<String>,
    pub failure_reason: Option<AgentLoopFailureReason>,
    pub pending_approval: Option<SuspendedToolApproval>,
    pub events: Vec<RuntimeEvent>,
    pub attempts: Vec<ModelInvocationAttempt>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComprehensiveApprovalContinuation {
    pub input: ComprehensiveTurnInput,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub call: ToolCallRequest,
    pub next_model_turn_index: u32,
    pub completed_tool_calls: u32,
    pub observation_bytes: u64,
    pub consecutive_invalid_tool_calls: u32,
    pub continuation_messages: Vec<ModelMessage>,
    pending_actions: VecDeque<QueuedResourceAction>,
    pub attempts: Vec<ModelInvocationAttempt>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingComprehensiveApproval {
    pub suspended: SuspendedToolApproval,
    pub continuation: ComprehensiveApprovalContinuation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfiguredTurnOutcome {
    pub final_message: String,
    pub selected_provider: String,
    pub selected_model: String,
    pub events: Vec<RuntimeEvent>,
    pub attempts: Vec<ModelInvocationAttempt>,
}

pub type ConfiguredTurnEventStream =
    Pin<Box<dyn Stream<Item = Result<RuntimeEvent, ConfiguredTurnError>> + Send>>;

pub struct ConfiguredTurnStream {
    pub selected_provider: String,
    pub selected_model: String,
    pub attempts: Vec<ModelInvocationAttempt>,
    pub events: ConfiguredTurnEventStream,
    attempt_log: Arc<Mutex<Vec<ModelInvocationAttempt>>>,
}

#[derive(Clone, Debug, Default)]
pub struct ConfiguredTurnStreamOptions {
    pub cancellation: CancellationToken,
    /// Shared tool definitions; cloned by Arc pointer across agent-loop turns.
    pub tools: Arc<Vec<ModelToolDefinition>>,
    pub reasoning_effort: Option<ReasoningEffort>,
    /// When true, load AGENTS.md/CLAUDE.md + git status into the turn prompt.
    pub coding_context: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ComprehensiveTurnStreamOptions {
    pub cancellation: CancellationToken,
    pub reasoning_effort: Option<ReasoningEffort>,
    /// When true, load AGENTS.md/CLAUDE.md + git status into each model turn.
    pub coding_context: bool,
}

pub struct ConfiguredTurnRunner;

pub struct ComprehensiveTurnRunner;

impl ComprehensiveTurnRunner {
    pub async fn run(
        input: ComprehensiveTurnInput,
        registry: &ModelProviderRegistry,
    ) -> Result<AgentLoopOutcome, ConfiguredTurnError> {
        let configured = ConfiguredTurnInput::from_shared(
            input.ids,
            input.user_message,
            input.conversation_history,
            input.effective_config,
            input.requirements,
        );
        let stream = ConfiguredTurnRunner::run_stream(configured, registry).await?;
        collect_comprehensive_turn_stream(stream).await
    }

    pub async fn run_with_resources<P>(
        input: ComprehensiveTurnInput,
        registry: &ModelProviderRegistry,
        resources: ComprehensiveTurnResources<'_, P>,
    ) -> Result<AgentLoopOutcome, ConfiguredTurnError>
    where
        P: PolicyEngine + ?Sized,
    {
        let mut sink = |_event: RuntimeEvent| Ok(());
        let mut approval_sink = |_approval: PendingComprehensiveApproval| Ok(());
        run_comprehensive_resource_loop(
            input,
            registry,
            model_tools_from_registry(resources.tools),
            |ids, _model_turn_index, call| {
                // ToolLoop::run offloads execute onto the blocking pool when a
                // multi-thread Tokio runtime is active (interactive TUI path).
                ToolLoop
                    .run(ids, call, resources.tools, resources.policy)
                    .map_err(tool_execution_error)
            },
            ComprehensiveTurnStreamOptions::default(),
            &mut sink,
            &mut approval_sink,
        )
        .await
    }

    pub async fn run_with_resources_streaming<P, S>(
        input: ComprehensiveTurnInput,
        registry: &ModelProviderRegistry,
        resources: ComprehensiveTurnResources<'_, P>,
        options: ComprehensiveTurnStreamOptions,
        mut event_sink: S,
    ) -> Result<AgentLoopOutcome, ConfiguredTurnError>
    where
        P: PolicyEngine + ?Sized,
        S: FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
    {
        let mut approval_sink = |_approval: PendingComprehensiveApproval| Ok(());
        Self::run_with_resources_streaming_and_approval_sink(
            input,
            registry,
            resources,
            options,
            &mut event_sink,
            &mut approval_sink,
        )
        .await
    }

    pub async fn run_with_resources_streaming_and_approval_sink<P, S, A>(
        input: ComprehensiveTurnInput,
        registry: &ModelProviderRegistry,
        resources: ComprehensiveTurnResources<'_, P>,
        options: ComprehensiveTurnStreamOptions,
        mut event_sink: S,
        mut approval_sink: A,
    ) -> Result<AgentLoopOutcome, ConfiguredTurnError>
    where
        P: PolicyEngine + ?Sized,
        S: FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
        A: FnMut(PendingComprehensiveApproval) -> Result<(), ConfiguredTurnError>,
    {
        run_comprehensive_resource_loop(
            input,
            registry,
            model_tools_from_registry(resources.tools),
            |ids, _model_turn_index, call| {
                ToolLoop
                    .run(ids, call, resources.tools, resources.policy)
                    .map_err(tool_execution_error)
            },
            options,
            &mut event_sink,
            &mut approval_sink,
        )
        .await
    }

    pub async fn continue_after_approval_streaming<P, S, A>(
        continuation: ComprehensiveApprovalContinuation,
        approval_outcome: ApprovalResumeOutcome,
        registry: &ModelProviderRegistry,
        resources: ComprehensiveTurnResources<'_, P>,
        options: ComprehensiveTurnStreamOptions,
        mut event_sink: S,
        mut approval_sink: A,
    ) -> Result<AgentLoopOutcome, ConfiguredTurnError>
    where
        P: PolicyEngine + ?Sized,
        S: FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
        A: FnMut(PendingComprehensiveApproval) -> Result<(), ConfiguredTurnError>,
    {
        let state = ComprehensiveLoopState::from_approval_outcome(continuation, approval_outcome)?;
        run_comprehensive_resource_loop_from_state(
            state,
            registry,
            model_tools_from_registry(resources.tools),
            |ids, _model_turn_index, call| {
                ToolLoop
                    .run(ids, call, resources.tools, resources.policy)
                    .map_err(tool_execution_error)
            },
            options,
            &mut event_sink,
            &mut approval_sink,
        )
        .await
    }

    pub async fn run_with_idempotency<P, S>(
        input: ComprehensiveTurnInput,
        registry: &ModelProviderRegistry,
        resources: ComprehensiveTurnResources<'_, P>,
        effects: &mut S,
    ) -> Result<AgentLoopOutcome, ConfiguredTurnError>
    where
        P: PolicyEngine + ?Sized,
        S: IdempotencyStore + ?Sized,
    {
        let executor = IdempotentToolExecutor::default();
        let mut sink = |_event: RuntimeEvent| Ok(());
        let mut approval_sink = |_approval: PendingComprehensiveApproval| Ok(());
        run_comprehensive_resource_loop(
            input,
            registry,
            model_tools_from_registry(resources.tools),
            |ids, model_turn_index, call| {
                let key = comprehensive_idempotency_key(ids, model_turn_index, &call)?;
                executor
                    .run(ids, key, call, resources.tools, resources.policy, effects)
                    .map(|outcome| outcome.tool)
                    .map_err(|error| ConfiguredTurnError::InvalidInput {
                        field: "idempotency".to_string(),
                        message: error.to_string(),
                    })
            },
            ComprehensiveTurnStreamOptions::default(),
            &mut sink,
            &mut approval_sink,
        )
        .await
    }
}

impl ConfiguredTurnRunner {
    pub async fn run(
        input: ConfiguredTurnInput,
        registry: &ModelProviderRegistry,
    ) -> Result<ConfiguredTurnOutcome, ConfiguredTurnError> {
        let stream = Self::run_stream(input, registry).await?;
        collect_configured_turn_stream(stream).await
    }

    pub async fn run_stream(
        input: ConfiguredTurnInput,
        registry: &ModelProviderRegistry,
    ) -> Result<ConfiguredTurnStream, ConfiguredTurnError> {
        Self::run_stream_with_options(input, registry, ConfiguredTurnStreamOptions::default()).await
    }

    pub async fn run_stream_with_options(
        input: ConfiguredTurnInput,
        registry: &ModelProviderRegistry,
        options: ConfiguredTurnStreamOptions,
    ) -> Result<ConfiguredTurnStream, ConfiguredTurnError> {
        Self::run_stream_with_continuation(input, registry, options, Vec::new()).await
    }

    async fn run_stream_with_continuation(
        input: ConfiguredTurnInput,
        registry: &ModelProviderRegistry,
        options: ConfiguredTurnStreamOptions,
        continuation_messages: Vec<ModelMessage>,
    ) -> Result<ConfiguredTurnStream, ConfiguredTurnError> {
        let prompt_snapshot_event = prompt_snapshot_event(
            input.ids,
            &input.user_message,
            input.conversation_history.as_ref(),
        )?;
        let runtime =
            ModelRuntimeResolver::resolve(&input.effective_config, registry, &input.requirements)?;
        let attempts = vec![attempt(&runtime.provider_id, &runtime.model, 1)];
        let policy_provider = Arc::new(PolicyModelProvider::new(
            runtime.clone(),
            registry.clone(),
            input.requirements.clone(),
        ));
        let attempt_log = Arc::clone(&policy_provider.attempts);
        let turn_input = TurnInput {
            ids: input.ids,
            user_message: input.user_message,
            // One history clone per model turn is required for TurnEngine ownership;
            // the Arc on ConfiguredTurnInput avoids re-cloning into each ConfiguredTurnInput.
            conversation_history: input.conversation_history.as_ref().clone(),
            continuation_messages,
            provider: runtime.provider_id.clone(),
            model: runtime.model.clone(),
        };
        let error_attempts = attempts.clone();
        // Prefer explicit empty assembler for deterministic unit tests; coding
        // hosts inject workspace layers via `TurnEngine::for_workspace` only when
        // AGENT_CODING_CONTEXT=1 or when building from AgentSession (below).
        let turn_engine = if options.coding_context {
            let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            TurnEngine::for_workspace(&root)
        } else {
            TurnEngine::default()
        };
        let turn_events = turn_engine.run_stream_with_options(
            turn_input,
            policy_provider,
            TurnStreamOptions {
                cancellation: options.cancellation,
                tools: options.tools.as_ref().clone(),
                reasoning_effort: options.reasoning_effort,
            },
        );
        let events = stream::iter([Ok(prompt_snapshot_event)]).chain(turn_events.map(
            move |event| match event {
                Ok(event) => Ok(event),
                Err(TurnError::Model { message, .. }) => Err(ConfiguredTurnError::Model {
                    message,
                    events: Vec::new(),
                    attempts: error_attempts.clone(),
                }),
            },
        ));

        Ok(ConfiguredTurnStream {
            selected_provider: runtime.provider_id,
            selected_model: runtime.model,
            attempts,
            events: Box::pin(events),
            attempt_log,
        })
    }

    async fn run_with_prompt_snapshot_event(
        input: ConfiguredTurnInput,
        registry: &ModelProviderRegistry,
        prompt_snapshot_event: RuntimeEvent,
    ) -> Result<ConfiguredTurnOutcome, ConfiguredTurnError> {
        if input.user_message.trim().is_empty() {
            return Err(ConfiguredTurnError::InvalidInput {
                field: "user_message".to_string(),
                message: "user message must not be empty".to_string(),
            });
        }

        let runtime =
            ModelRuntimeResolver::resolve(&input.effective_config, registry, &input.requirements)?;
        let policy_provider = PolicyModelProvider::new(
            runtime.clone(),
            registry.clone(),
            input.requirements.clone(),
        );
        let turn_input = TurnInput {
            ids: input.ids,
            user_message: input.user_message,
            conversation_history: input.conversation_history.as_ref().clone(),
            continuation_messages: Vec::new(),
            provider: runtime.provider_id.clone(),
            model: runtime.model.clone(),
        };

        match TurnEngine::default()
            .run(turn_input, &policy_provider)
            .await
        {
            Ok(outcome) => {
                let events = prepend_event(prompt_snapshot_event, outcome.events);
                Ok(ConfiguredTurnOutcome {
                    final_message: outcome.final_message,
                    selected_provider: runtime.provider_id,
                    selected_model: runtime.model,
                    events,
                    attempts: policy_provider.attempts(),
                })
            }
            Err(TurnError::Model { message, events }) => {
                let events = prepend_event(prompt_snapshot_event, events);
                Err(ConfiguredTurnError::Model {
                    message,
                    events,
                    attempts: policy_provider.attempts(),
                })
            }
        }
    }

    pub async fn run_and_persist<S>(
        input: ConfiguredTurnInput,
        registry: &ModelProviderRegistry,
        store: &mut S,
    ) -> Result<ConfiguredTurnOutcome, ConfiguredTurnPersistenceError>
    where
        S: EventStore,
    {
        let prompt_snapshot_event = prompt_snapshot_event(
            input.ids,
            &input.user_message,
            input.conversation_history.as_ref(),
        )?;
        append_events(store, std::slice::from_ref(&prompt_snapshot_event))?;

        match Self::run_with_prompt_snapshot_event(input, registry, prompt_snapshot_event).await {
            Ok(outcome) => {
                append_events(store, tail_events(&outcome.events))?;
                Ok(outcome)
            }
            Err(error) => {
                if let ConfiguredTurnError::Model { events, .. } = &error {
                    append_events(store, tail_events(events))?;
                }
                Err(ConfiguredTurnPersistenceError::Turn(error))
            }
        }
    }
}

async fn run_comprehensive_resource_loop<F>(
    input: ComprehensiveTurnInput,
    registry: &ModelProviderRegistry,
    model_tools: Arc<Vec<ModelToolDefinition>>,
    execute_tool: F,
    options: ComprehensiveTurnStreamOptions,
    event_sink: &mut impl FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
    approval_sink: &mut impl FnMut(PendingComprehensiveApproval) -> Result<(), ConfiguredTurnError>,
) -> Result<AgentLoopOutcome, ConfiguredTurnError>
where
    F: FnMut(
        RuntimeIds,
        u32,
        ToolCallRequest,
    ) -> Result<crate::ToolLoopOutcome, ConfiguredTurnError>,
{
    run_comprehensive_resource_loop_from_state(
        ComprehensiveLoopState::new(input, options.reasoning_effort),
        registry,
        model_tools,
        execute_tool,
        options,
        event_sink,
        approval_sink,
    )
    .await
}

async fn run_comprehensive_resource_loop_from_state<F>(
    mut state: ComprehensiveLoopState,
    registry: &ModelProviderRegistry,
    model_tools: Arc<Vec<ModelToolDefinition>>,
    mut execute_tool: F,
    options: ComprehensiveTurnStreamOptions,
    event_sink: &mut impl FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
    approval_sink: &mut impl FnMut(PendingComprehensiveApproval) -> Result<(), ConfiguredTurnError>,
) -> Result<AgentLoopOutcome, ConfiguredTurnError>
where
    F: FnMut(
        RuntimeIds,
        u32,
        ToolCallRequest,
    ) -> Result<crate::ToolLoopOutcome, ConfiguredTurnError>,
{
    if invalid_loop_budget(&state.input.loop_budget) {
        return failed_loop_outcome_streaming(
            state.input.ids,
            state.all_events,
            state.attempts,
            AgentLoopFailureReason::InvalidBudget,
            event_sink,
        );
    }

    loop {
        if wall_time_exceeded(&state.started_at, state.input.loop_budget.max_wall_time_ms) {
            return failed_loop_outcome_streaming(
                state.input.ids,
                state.all_events,
                state.attempts,
                AgentLoopFailureReason::MaxWallTimeExceeded,
                event_sink,
            );
        }
        let pending_action = state.pending_actions.pop_front();
        let (model_turn_index, resource_turn) = if let Some(pending) = pending_action {
            (
                pending.model_turn_index,
                match pending.action {
                    ResourceModelAction::ToolCall(call) => ResourceModelTurn::ToolCall {
                        call,
                        events: Vec::new(),
                        attempts: Vec::new(),
                    },
                    ResourceModelAction::Observation { call, observation } => {
                        ResourceModelTurn::Observation {
                            call,
                            observation,
                            events: Vec::new(),
                            attempts: Vec::new(),
                        }
                    }
                },
            )
        } else {
            let model_turn_index = state.next_model_turn_index;
            state.next_model_turn_index = state.next_model_turn_index.saturating_add(1);
            if model_turn_index > 0
                && model_turn_index % state.input.loop_budget.model_turn_checkpoint_interval == 0
            {
                let warning = runtime_warning_event(
                    state.input.ids,
                    "model_turn_budget_extended",
                    "model turn soft limit reached; continuing while hard budgets remain",
                );
                event_sink(warning.clone())?;
                state.all_events.push(warning);
            }
            // Arc::clone on shared history/config/tools — pointer bumps only.
            let configured = ConfiguredTurnInput::from_shared(
                state.input.ids,
                state.input.user_message.clone(),
                Arc::clone(&state.input.conversation_history),
                Arc::clone(&state.input.effective_config),
                state.input.requirements.clone(),
            );
            let stream = ConfiguredTurnRunner::run_stream_with_continuation(
                configured,
                registry,
                ConfiguredTurnStreamOptions {
                    cancellation: options.cancellation.clone(),
                    tools: Arc::clone(&model_tools),
                    reasoning_effort: state.reasoning_effort,
                    coding_context: options.coding_context,
                },
                state.continuation_messages.clone(),
            )
            .await?;
            let resource_turn =
                collect_resource_model_turn_streaming(stream, model_turn_index == 0, event_sink)
                    .await?;
            (model_turn_index, resource_turn)
        };

        match resource_turn {
            ResourceModelTurn::Final {
                final_message,
                events,
                attempts,
            } => {
                state.all_events.extend(events);
                state.extend_attempts(attempts);
                return Ok(AgentLoopOutcome {
                    terminal_state: AgentLoopTerminalState::Done,
                    final_message,
                    failure_reason: None,
                    pending_approval: None,
                    events: state.all_events,
                    attempts: state.attempts,
                });
            }
            ResourceModelTurn::Cancelled { events, attempts } => {
                state.all_events.extend(events);
                state.extend_attempts(attempts);
                return Ok(AgentLoopOutcome {
                    terminal_state: AgentLoopTerminalState::Cancelled,
                    final_message: None,
                    failure_reason: None,
                    pending_approval: None,
                    events: state.all_events,
                    attempts: state.attempts,
                });
            }
            ResourceModelTurn::Actions {
                actions,
                events,
                attempts,
            } => {
                state.all_events.extend(events);
                state.extend_attempts(attempts);
                state
                    .pending_actions
                    .extend(actions.into_iter().map(|action| QueuedResourceAction {
                        model_turn_index,
                        action,
                    }));
                continue;
            }
            ResourceModelTurn::ToolCall {
                call,
                events,
                attempts,
            } => {
                state.all_events.extend(events);
                state.extend_attempts(attempts);
                if state.completed_tool_calls >= state.input.loop_budget.max_tool_calls {
                    return failed_loop_outcome_streaming(
                        state.input.ids,
                        state.all_events,
                        state.attempts,
                        AgentLoopFailureReason::MaxToolCallsExceeded,
                        event_sink,
                    );
                }
                if wall_time_exceeded(&state.started_at, state.input.loop_budget.max_wall_time_ms) {
                    return failed_loop_outcome_streaming(
                        state.input.ids,
                        state.all_events,
                        state.attempts,
                        AgentLoopFailureReason::MaxWallTimeExceeded,
                        event_sink,
                    );
                }
                state.completed_tool_calls = state.completed_tool_calls.saturating_add(1);
                let tool_outcome = execute_tool(state.input.ids, model_turn_index, call.clone())?;
                let status = tool_outcome.status;
                state.push_assistant_tool_call(&call);
                let observation = observation_from_tool_outcome(
                    &call,
                    status,
                    tool_outcome.observation.as_deref(),
                    state.input.loop_budget.max_observation_bytes,
                );
                if status == ToolLoopStatus::WaitingApproval {
                    let pending_approval = tool_outcome.pending_approval.clone().map(|approval| {
                        SuspendedToolApproval {
                            ids: state.input.ids,
                            model_turn_index,
                            call: call.clone(),
                            approval,
                        }
                    });
                    if let Some(pending) = pending_approval.clone() {
                        approval_sink(PendingComprehensiveApproval {
                            suspended: pending,
                            continuation: state.continuation_for_waiting_call(call),
                        })?;
                    }
                    record_events_streaming(
                        &mut state.all_events,
                        tool_outcome.events,
                        event_sink,
                    )?;
                    return Ok(AgentLoopOutcome {
                        terminal_state: AgentLoopTerminalState::WaitingApproval,
                        final_message: None,
                        failure_reason: None,
                        pending_approval,
                        events: state.all_events,
                        attempts: state.attempts,
                    });
                }
                record_events_streaming(&mut state.all_events, tool_outcome.events, event_sink)?;

                state.push_tool_observation(&call, observation);
                if state.observation_bytes > state.input.loop_budget.max_observation_bytes {
                    return failed_loop_outcome_streaming(
                        state.input.ids,
                        state.all_events,
                        state.attempts,
                        AgentLoopFailureReason::MaxObservationBytesExceeded,
                        event_sink,
                    );
                }
            }
            ResourceModelTurn::Observation {
                call,
                observation,
                events,
                attempts,
            } => {
                state.all_events.extend(events);
                state.extend_attempts(attempts);
                state.consecutive_invalid_tool_calls =
                    state.consecutive_invalid_tool_calls.saturating_add(1);
                if state.consecutive_invalid_tool_calls
                    > state.input.loop_budget.max_consecutive_invalid_tool_calls
                {
                    return failed_loop_outcome_streaming(
                        state.input.ids,
                        state.all_events,
                        state.attempts,
                        AgentLoopFailureReason::MaxConsecutiveInvalidToolCallsExceeded,
                        event_sink,
                    );
                }
                state.push_assistant_tool_call(&call);
                state.push_invalid_observation(&call, observation);
                if state.observation_bytes > state.input.loop_budget.max_observation_bytes {
                    return failed_loop_outcome_streaming(
                        state.input.ids,
                        state.all_events,
                        state.attempts,
                        AgentLoopFailureReason::MaxObservationBytesExceeded,
                        event_sink,
                    );
                }
            }
        }
    }
}

fn model_tools_from_registry(registry: &ToolRegistry) -> Arc<Vec<ModelToolDefinition>> {
    Arc::new(
        registry
            .definitions()
            .into_iter()
            .map(|definition| ModelToolDefinition {
                name: definition.name.as_str().to_string(),
                description: definition.description,
                input_schema: definition.input_schema,
            })
            .collect(),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ResourceModelAction {
    ToolCall(ToolCallRequest),
    Observation {
        call: ToolCallRequest,
        observation: ToolObservationMessage,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct QueuedResourceAction {
    model_turn_index: u32,
    action: ResourceModelAction,
}

#[derive(Debug)]
struct ComprehensiveLoopState {
    input: ComprehensiveTurnInput,
    reasoning_effort: Option<ReasoningEffort>,
    next_model_turn_index: u32,
    completed_tool_calls: u32,
    observation_bytes: u64,
    consecutive_invalid_tool_calls: u32,
    continuation_messages: Vec<ModelMessage>,
    pending_actions: VecDeque<QueuedResourceAction>,
    all_events: Vec<RuntimeEvent>,
    attempts: Vec<ModelInvocationAttempt>,
    started_at: Instant,
}

impl ComprehensiveLoopState {
    fn new(input: ComprehensiveTurnInput, reasoning_effort: Option<ReasoningEffort>) -> Self {
        Self {
            input,
            reasoning_effort,
            next_model_turn_index: 0,
            completed_tool_calls: 0,
            observation_bytes: 0,
            consecutive_invalid_tool_calls: 0,
            continuation_messages: Vec::new(),
            pending_actions: VecDeque::new(),
            all_events: Vec::new(),
            attempts: Vec::new(),
            started_at: Instant::now(),
        }
    }

    fn from_approval_outcome(
        continuation: ComprehensiveApprovalContinuation,
        approval_outcome: ApprovalResumeOutcome,
    ) -> Result<Self, ConfiguredTurnError> {
        let call = continuation.call.clone();
        let mut state = Self {
            input: continuation.input,
            reasoning_effort: continuation.reasoning_effort,
            next_model_turn_index: continuation.next_model_turn_index,
            completed_tool_calls: continuation.completed_tool_calls,
            observation_bytes: continuation.observation_bytes,
            consecutive_invalid_tool_calls: continuation.consecutive_invalid_tool_calls,
            continuation_messages: continuation.continuation_messages,
            pending_actions: continuation.pending_actions,
            all_events: Vec::new(),
            attempts: continuation.attempts,
            started_at: Instant::now(),
        };
        let (status, observation) = match approval_outcome.tool.as_ref() {
            Some(tool) => (tool.status, tool.observation.as_deref()),
            None => (ToolLoopStatus::Denied, None),
        };
        let observation = observation_from_tool_outcome(
            &call,
            status,
            observation,
            state.input.loop_budget.max_observation_bytes,
        );
        state.push_tool_observation(&call, observation);
        if state.observation_bytes > state.input.loop_budget.max_observation_bytes {
            return Err(ConfiguredTurnError::InvalidInput {
                field: "approval_observation".to_string(),
                message: "tool observation byte budget was exhausted".to_string(),
            });
        }
        Ok(state)
    }

    fn continuation_for_waiting_call(
        &self,
        call: ToolCallRequest,
    ) -> ComprehensiveApprovalContinuation {
        ComprehensiveApprovalContinuation {
            input: self.input.clone(),
            reasoning_effort: self.reasoning_effort,
            call,
            next_model_turn_index: self.next_model_turn_index,
            completed_tool_calls: self.completed_tool_calls,
            observation_bytes: self.observation_bytes,
            consecutive_invalid_tool_calls: self.consecutive_invalid_tool_calls,
            continuation_messages: self.continuation_messages.clone(),
            pending_actions: self.pending_actions.clone(),
            attempts: self.attempts.clone(),
        }
    }

    fn extend_attempts(&mut self, attempts: Vec<ModelInvocationAttempt>) {
        self.attempts.extend(attempts);
    }

    fn push_assistant_tool_call(&mut self, call: &ToolCallRequest) {
        self.continuation_messages
            .push(ModelMessage::assistant_tool_call(
                call.call_id.clone(),
                call.tool_name.clone(),
                call.input_json.clone(),
            ));
    }

    fn push_tool_observation(
        &mut self,
        call: &ToolCallRequest,
        observation: ToolObservationMessage,
    ) {
        self.consecutive_invalid_tool_calls = 0;
        self.observation_bytes = self
            .observation_bytes
            .saturating_add(observation.observation.len() as u64);
        self.continuation_messages.push(ModelMessage::tool_result(
            call.call_id.clone(),
            call.tool_name.clone(),
            observation.observation,
        ));
    }

    fn push_invalid_observation(
        &mut self,
        call: &ToolCallRequest,
        observation: ToolObservationMessage,
    ) {
        self.observation_bytes = self
            .observation_bytes
            .saturating_add(observation.observation.len() as u64);
        self.continuation_messages.push(ModelMessage::tool_result(
            call.call_id.clone(),
            call.tool_name.clone(),
            observation.observation,
        ));
    }
}

fn invalid_loop_budget(budget: &AgentLoopBudget) -> bool {
    budget.model_turn_checkpoint_interval == 0
        || budget.max_tool_calls == 0
        || budget.max_wall_time_ms == 0
        || budget.max_observation_bytes == 0
        || budget.max_consecutive_invalid_tool_calls == 0
}

fn wall_time_exceeded(started_at: &Instant, max_wall_time_ms: u64) -> bool {
    started_at.elapsed().as_millis() > u128::from(max_wall_time_ms)
}

fn failed_loop_outcome_streaming(
    ids: RuntimeIds,
    mut events: Vec<RuntimeEvent>,
    attempts: Vec<ModelInvocationAttempt>,
    reason: AgentLoopFailureReason,
    event_sink: &mut impl FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
) -> Result<AgentLoopOutcome, ConfiguredTurnError> {
    let event = runtime_error_event(ids, &reason);
    event_sink(event.clone())?;
    events.push(event);
    Ok(AgentLoopOutcome {
        terminal_state: AgentLoopTerminalState::Failed,
        final_message: None,
        failure_reason: Some(reason),
        pending_approval: None,
        events,
        attempts,
    })
}

fn record_events_streaming(
    all_events: &mut Vec<RuntimeEvent>,
    events: Vec<RuntimeEvent>,
    event_sink: &mut impl FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
) -> Result<(), ConfiguredTurnError> {
    for event in events {
        event_sink(event.clone())?;
        all_events.push(event);
    }
    Ok(())
}

fn runtime_warning_event(ids: RuntimeIds, code: &str, message: &str) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: ruse_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: ids.workspace_id,
        conversation_id: ids.conversation_id,
        turn_id: Some(ids.turn_id),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: ids.correlation_id,
        causation_id: None,
        payload: RuntimeEventPayload::RuntimeWarning(ruse_types::RuntimeWarning {
            code: code.to_string(),
            message: message.to_string(),
        }),
    }
}

fn runtime_error_event(ids: RuntimeIds, reason: &AgentLoopFailureReason) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: ruse_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: ids.workspace_id,
        conversation_id: ids.conversation_id,
        turn_id: Some(ids.turn_id),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: ids.correlation_id,
        causation_id: None,
        payload: RuntimeEventPayload::RuntimeError(RuntimeError {
            code: failure_reason_code(reason).to_string(),
            message: failure_reason_message(reason),
            recoverable: false,
        }),
    }
}

fn failure_reason_code(reason: &AgentLoopFailureReason) -> &'static str {
    match reason {
        AgentLoopFailureReason::MaxToolCallsExceeded => "max_tool_calls_exceeded",
        AgentLoopFailureReason::MaxWallTimeExceeded => "max_wall_time_exceeded",
        AgentLoopFailureReason::MaxObservationBytesExceeded => "max_observation_bytes_exceeded",
        AgentLoopFailureReason::MaxConsecutiveInvalidToolCallsExceeded => {
            "max_consecutive_invalid_tool_calls_exceeded"
        }
        AgentLoopFailureReason::InvalidBudget => "invalid_loop_budget",
        AgentLoopFailureReason::Runtime(_) => "agent_loop_runtime_error",
    }
}

fn failure_reason_message(reason: &AgentLoopFailureReason) -> String {
    match reason {
        AgentLoopFailureReason::MaxToolCallsExceeded => {
            "tool call budget was exhausted".to_string()
        }
        AgentLoopFailureReason::MaxWallTimeExceeded => {
            "wall-clock budget was exhausted".to_string()
        }
        AgentLoopFailureReason::MaxObservationBytesExceeded => {
            "tool observation byte budget was exhausted".to_string()
        }
        AgentLoopFailureReason::MaxConsecutiveInvalidToolCallsExceeded => {
            "invalid tool-call proposal budget was exhausted".to_string()
        }
        AgentLoopFailureReason::InvalidBudget => "loop budget values must be non-zero".to_string(),
        AgentLoopFailureReason::Runtime(message) => message.clone(),
    }
}

fn tool_execution_error(error: crate::ToolLoopError) -> ConfiguredTurnError {
    ConfiguredTurnError::InvalidInput {
        field: "tool_call".to_string(),
        message: error.to_string(),
    }
}

fn comprehensive_idempotency_key(
    ids: RuntimeIds,
    model_turn_index: u32,
    call: &ToolCallRequest,
) -> Result<IdempotencyKey, ConfiguredTurnError> {
    let input_hash = stable_tool_input_hash(call);
    IdempotencyKey::new(format!(
        "turn:{}:{}:{}:{}:{}:{}:{}",
        ids.workspace_id,
        ids.conversation_id,
        ids.turn_id,
        model_turn_index,
        call.call_id,
        call.tool_name,
        input_hash
    ))
    .map_err(|error| ConfiguredTurnError::InvalidInput {
        field: "idempotency_key".to_string(),
        message: error.to_string(),
    })
}

fn stable_tool_input_hash(call: &ToolCallRequest) -> String {
    let input =
        serde_json::to_string(&call.input_json).unwrap_or_else(|_| call.input_json.to_string());
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv1a(hash, call.tool_name.as_bytes());
    hash = fnv1a(hash, b"\0");
    hash = fnv1a(hash, input.as_bytes());
    format!("fnv1a64:{hash:016x}")
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

enum ResourceModelTurn {
    Final {
        final_message: Option<String>,
        events: Vec<RuntimeEvent>,
        attempts: Vec<ModelInvocationAttempt>,
    },
    Cancelled {
        events: Vec<RuntimeEvent>,
        attempts: Vec<ModelInvocationAttempt>,
    },
    Actions {
        actions: Vec<ResourceModelAction>,
        events: Vec<RuntimeEvent>,
        attempts: Vec<ModelInvocationAttempt>,
    },
    // Queued actions re-enter the existing single-call execution branches so
    // approval and budget accounting remain identical for every proposal.
    ToolCall {
        call: ToolCallRequest,
        events: Vec<RuntimeEvent>,
        attempts: Vec<ModelInvocationAttempt>,
    },
    Observation {
        call: ToolCallRequest,
        observation: ToolObservationMessage,
        events: Vec<RuntimeEvent>,
        attempts: Vec<ModelInvocationAttempt>,
    },
}

fn is_synthetic_tool_turn_completion(payload: &RuntimeEventPayload) -> bool {
    match payload {
        RuntimeEventPayload::FinalResponseProduced(_) => true,
        RuntimeEventPayload::TurnCompleted(completed) => completed.final_state == TurnState::Done,
        RuntimeEventPayload::TurnStateChanged(changed) => {
            matches!(changed.to, TurnState::Finalizing | TurnState::Done)
        }
        _ => false,
    }
}

async fn collect_resource_model_turn_streaming(
    turn_stream: ConfiguredTurnStream,
    emit_user_input: bool,
    event_sink: &mut impl FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
) -> Result<ResourceModelTurn, ConfiguredTurnError> {
    let ConfiguredTurnStream {
        attempts,
        attempt_log,
        mut events,
        ..
    } = turn_stream;
    let mut collected_events = Vec::new();
    let mut final_message = None;
    let mut cancelled = false;
    let mut actions = Vec::new();
    let mut accumulator = ToolCallAccumulator::default();

    while let Some(event) = events.next().await {
        match event {
            Ok(event) => {
                if !emit_user_input
                    && matches!(&event.payload, RuntimeEventPayload::UserInputReceived(_))
                {
                    continue;
                }
                if !actions.is_empty() && is_synthetic_tool_turn_completion(&event.payload) {
                    continue;
                }
                match &event.payload {
                    RuntimeEventPayload::ToolCallProposed(proposed) => {
                        let call = ToolCallRequest {
                            call_id: proposed.call_id.clone(),
                            tool_name: proposed.tool_name.clone(),
                            input_json: proposed.input_json.clone(),
                        };
                        let accumulated = accumulator.push_tool_call(
                            call.call_id.clone(),
                            call.tool_name.clone(),
                            call.input_json.clone(),
                        );
                        match accumulated {
                            AccumulatedToolCall::Ready(call) => {
                                actions.push(ResourceModelAction::ToolCall(call));
                            }
                            AccumulatedToolCall::Observation(observation) => {
                                actions
                                    .push(ResourceModelAction::Observation { call, observation });
                            }
                        }
                    }
                    RuntimeEventPayload::FinalResponseProduced(response) => {
                        final_message = Some(response.message.clone());
                    }
                    RuntimeEventPayload::TurnCompleted(completed)
                        if completed.final_state == TurnState::Cancelled =>
                    {
                        cancelled = true;
                    }
                    _ => {}
                }
                event_sink(event.clone())?;
                collected_events.push(event);
            }
            Err(ConfiguredTurnError::Model { message, .. }) => {
                let attempts = logged_attempts_or_initial(&attempt_log, attempts);
                return Err(ConfiguredTurnError::Model {
                    message,
                    events: collected_events,
                    attempts,
                });
            }
            Err(error) => return Err(error),
        }
    }

    let attempts = logged_attempts_or_initial(&attempt_log, attempts);
    if cancelled {
        Ok(ResourceModelTurn::Cancelled {
            events: collected_events,
            attempts,
        })
    } else if !actions.is_empty() {
        Ok(ResourceModelTurn::Actions {
            actions,
            events: collected_events,
            attempts,
        })
    } else {
        Ok(ResourceModelTurn::Final {
            final_message,
            events: collected_events,
            attempts,
        })
    }
}

async fn collect_comprehensive_turn_stream(
    turn_stream: ConfiguredTurnStream,
) -> Result<AgentLoopOutcome, ConfiguredTurnError> {
    let ConfiguredTurnStream {
        attempts,
        attempt_log,
        mut events,
        ..
    } = turn_stream;
    let mut collected_events = Vec::new();
    let mut final_message = None;

    while let Some(event) = events.next().await {
        match event {
            Ok(event) => {
                match &event.payload {
                    RuntimeEventPayload::ToolCallProposed(_) => {
                        collected_events.push(event);
                        return Ok(AgentLoopOutcome {
                            terminal_state: AgentLoopTerminalState::WaitingApproval,
                            final_message: None,
                            failure_reason: None,
                            pending_approval: None,
                            events: collected_events,
                            attempts: logged_attempts_or_initial(&attempt_log, attempts.clone()),
                        });
                    }
                    RuntimeEventPayload::FinalResponseProduced(response) => {
                        final_message = Some(response.message.clone());
                    }
                    _ => {}
                }
                collected_events.push(event);
            }
            Err(ConfiguredTurnError::Model { message, .. }) => {
                let attempts = logged_attempts_or_initial(&attempt_log, attempts);
                return Err(ConfiguredTurnError::Model {
                    message,
                    events: collected_events,
                    attempts,
                });
            }
            Err(error) => return Err(error),
        }
    }

    Ok(AgentLoopOutcome {
        terminal_state: AgentLoopTerminalState::Done,
        final_message,
        failure_reason: None,
        pending_approval: None,
        events: collected_events,
        attempts: logged_attempts_or_initial(&attempt_log, attempts),
    })
}

async fn collect_configured_turn_stream(
    turn_stream: ConfiguredTurnStream,
) -> Result<ConfiguredTurnOutcome, ConfiguredTurnError> {
    let ConfiguredTurnStream {
        selected_provider,
        selected_model,
        attempts,
        attempt_log,
        mut events,
    } = turn_stream;
    let mut collected_events = Vec::new();
    let mut final_message = String::new();

    while let Some(event) = events.next().await {
        match event {
            Ok(event) => {
                if let RuntimeEventPayload::FinalResponseProduced(response) = &event.payload {
                    final_message = response.message.clone();
                }
                collected_events.push(event);
            }
            Err(ConfiguredTurnError::Model { message, .. }) => {
                let attempts = logged_attempts_or_initial(&attempt_log, attempts);
                return Err(ConfiguredTurnError::Model {
                    message,
                    events: collected_events,
                    attempts,
                });
            }
            Err(error) => return Err(error),
        }
    }

    let attempts = logged_attempts_or_initial(&attempt_log, attempts);
    Ok(ConfiguredTurnOutcome {
        final_message,
        selected_provider,
        selected_model,
        events: collected_events,
        attempts,
    })
}

fn logged_attempts_or_initial(
    attempt_log: &Arc<Mutex<Vec<ModelInvocationAttempt>>>,
    initial_attempts: Vec<ModelInvocationAttempt>,
) -> Vec<ModelInvocationAttempt> {
    let attempts = attempt_log.lock().expect("attempts lock").clone();
    if attempts.is_empty() {
        initial_attempts
    } else {
        attempts
    }
}

fn prepend_event(first: RuntimeEvent, mut rest: Vec<RuntimeEvent>) -> Vec<RuntimeEvent> {
    let mut events = Vec::with_capacity(rest.len() + 1);
    events.push(first);
    events.append(&mut rest);
    events
}

fn observation_from_tool_outcome(
    call: &ToolCallRequest,
    status: ToolLoopStatus,
    observation: Option<&str>,
    max_observation_bytes: u64,
) -> ToolObservationMessage {
    let raw = match status {
        ToolLoopStatus::Completed => observation.unwrap_or("tool completed without output"),
        ToolLoopStatus::Denied => "tool call was denied by policy",
        ToolLoopStatus::WaitingApproval => "tool call is waiting for approval",
        ToolLoopStatus::Failed => "tool execution failed",
    };
    let (bounded, truncated) = truncate_utf8(&redact_secret_like(raw), max_observation_bytes);
    ToolObservationMessage {
        call_id: call.call_id.clone(),
        tool_name: call.tool_name.clone(),
        observation: format!(
            "tool observation for {} ({}): {}",
            call.tool_name, call.call_id, bounded
        ),
        truncated,
    }
}

fn truncate_utf8(input: &str, max_bytes: u64) -> (String, bool) {
    let max_bytes = usize::try_from(max_bytes).unwrap_or(usize::MAX);
    if input.len() <= max_bytes {
        return (input.to_string(), false);
    }
    if max_bytes == 0 {
        return (String::new(), true);
    }

    let mut end = max_bytes.min(input.len());
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }
    (input[..end].to_string(), true)
}

fn redact_secret_like(input: &str) -> String {
    input
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if part.starts_with("sk-")
                || lower.contains("api_key")
                || lower.contains("token")
                || lower.contains("password")
            {
                "[REDACTED]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn tail_events(events: &[RuntimeEvent]) -> &[RuntimeEvent] {
    events.get(1..).unwrap_or_default()
}

fn append_events<S>(
    store: &mut S,
    events: &[RuntimeEvent],
) -> Result<(), ConfiguredTurnPersistenceError>
where
    S: EventStore,
{
    for event in events {
        store
            .append(event.clone())
            .map_err(|error| ConfiguredTurnPersistenceError::Storage {
                message: error.to_string(),
            })?;
    }
    Ok(())
}

fn prompt_snapshot_event(
    ids: RuntimeIds,
    user_message: &str,
    conversation_history: &[PromptMessage],
) -> Result<RuntimeEvent, ConfiguredTurnError> {
    if user_message.trim().is_empty() {
        return Err(ConfiguredTurnError::InvalidInput {
            field: "user_message".to_string(),
            message: "user message must not be empty".to_string(),
        });
    }

    let profile = PromptProfile::new(
        PromptProfileId::new("default").expect("built-in profile id is valid"),
        1,
        "Default Agent",
        vec![ruse_context::PromptLayerId::new("system").expect("built-in layer id is valid")],
        PromptBudget::new(16_384, 4_096).expect("built-in prompt budget is valid"),
    )
    .expect("built-in prompt profile is valid");
    let workspace_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let workspace_instruction = crate::context::load_workspace_instruction_layer(&workspace_root)
        .map(|layer| layer.content)
        .unwrap_or_else(|| {
            "You are a local coding agent. Prefer small, reviewable edits. Use tools to inspect and change the workspace.".to_string()
        });
    let mut system_content =
        format!("agent runtime safety and integrity instructions\n\n{workspace_instruction}");
    if let Some(git) = crate::context::load_workspace_git_summary_layer(&workspace_root) {
        system_content.push_str("\n\n");
        system_content.push_str(&git.content);
    }
    let system_tokens = (system_content.len() as u32 / 4).max(8);
    let mut layers = vec![
        PromptLayer::new(
            "system",
            PromptLayerKind::Runtime,
            1_000,
            system_content,
            system_tokens,
            PromptLayerSource::BuiltIn,
            PromptLayerVisibility::Hidden,
            PromptLayerSensitivity::Public,
            PromptLayerPolicy::Required,
            1,
        )
        .expect("built-in system layer is valid"),
        PromptLayer::new(
            "user",
            PromptLayerKind::User,
            0,
            user_message,
            user_message.len() as u32,
            PromptLayerSource::SessionOverlay {
                command_id: "user_message".to_string(),
            },
            PromptLayerVisibility::Full,
            PromptLayerSensitivity::UserPrivate,
            PromptLayerPolicy::Optional,
            1,
        )
        .expect("validated non-empty user message creates a layer"),
    ];
    if let Some(history_layer) = prompt_snapshot_history_layer(conversation_history) {
        layers.insert(1, history_layer);
    }
    let outcome = PromptRuntime
        .assemble(PromptAssemblyRequest {
            profile,
            layers,
            overlays: Vec::new(),
        })
        .map_err(|error| ConfiguredTurnError::InvalidInput {
            field: "prompt".to_string(),
            message: error.to_string(),
        })?;
    let snapshot = PromptSnapshot::from_assembly(&outcome.bundle, &outcome.report);

    Ok(RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: ruse_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: ids.workspace_id,
        conversation_id: ids.conversation_id,
        turn_id: Some(ids.turn_id),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: ids.correlation_id,
        causation_id: None,
        payload: RuntimeEventPayload::PromptSnapshotRecorded(PromptSnapshotRecorded {
            profile_id: snapshot.profile_id.as_str().to_string(),
            profile_version: snapshot.profile_version,
            bundle_hash: snapshot.bundle_hash,
            layer_hashes: snapshot
                .layer_hashes
                .into_iter()
                .map(|hash| PromptLayerHashRecord {
                    layer_id: hash.layer_id.as_str().to_string(),
                    source_hash: hash.source_hash,
                    content_hash: hash.content_hash,
                })
                .collect(),
            diagnostic_count: snapshot.diagnostics.len() as u32,
            message_count: snapshot.message_count,
            estimated_tokens: snapshot.estimated_tokens,
        }),
    })
}

fn prompt_snapshot_history_layer(history: &[PromptMessage]) -> Option<PromptLayer> {
    if history.is_empty() {
        return None;
    }
    let content = format_prompt_snapshot_history(history);
    PromptLayer::new(
        "conversation_history",
        PromptLayerKind::Memory,
        100,
        content.clone(),
        content.len() as u32,
        PromptLayerSource::SessionOverlay {
            command_id: "conversation_history".to_string(),
        },
        PromptLayerVisibility::Full,
        PromptLayerSensitivity::UserPrivate,
        PromptLayerPolicy::Optional,
        1,
    )
    .ok()
}

fn format_prompt_snapshot_history(history: &[PromptMessage]) -> String {
    let mut output = String::from("Conversation so far:\n");
    for message in history {
        output.push_str(&message.role);
        output.push_str(": ");
        output.push_str(&message.content);
        output.push('\n');
    }
    output
}

struct PolicyModelProvider {
    runtime: ConfiguredModelRuntime,
    registry: ModelProviderRegistry,
    requirements: ModelRequestRequirements,
    attempts: Arc<Mutex<Vec<ModelInvocationAttempt>>>,
}

impl PolicyModelProvider {
    fn new(
        runtime: ConfiguredModelRuntime,
        registry: ModelProviderRegistry,
        requirements: ModelRequestRequirements,
    ) -> Self {
        Self {
            runtime,
            registry,
            requirements,
            attempts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn attempts(&self) -> Vec<ModelInvocationAttempt> {
        self.attempts.lock().expect("attempts lock").clone()
    }
}

#[async_trait::async_trait]
impl ModelProvider for PolicyModelProvider {
    async fn stream(
        &self,
        request: ModelRequest,
    ) -> Result<ruse_model::ModelEventStream, ModelError> {
        Ok(Box::pin(stream::unfold(
            StreamingInvocationState::new(
                self.runtime.clone(),
                self.registry.clone(),
                self.requirements.clone(),
                request,
                Arc::clone(&self.attempts),
            ),
            next_streaming_invocation_item,
        )))
    }
}

struct StreamingInvocationState {
    runtime: ConfiguredModelRuntime,
    registry: ModelProviderRegistry,
    requirements: ModelRequestRequirements,
    request_template: ModelRequest,
    attempts: Arc<Mutex<Vec<ModelInvocationAttempt>>>,
    selected_attempts_started: u32,
    fallback_index: usize,
    next_attempt_index: u32,
    use_model_override: bool,
    current_provider_id: Option<String>,
    current_model: Option<String>,
    current_attempt_position: Option<usize>,
    current_attempt_started_at: Option<Instant>,
    current_stream: Option<ruse_model::ModelEventStream>,
    crossed_side_effect_boundary: bool,
    last_failure: Option<InvocationFailure>,
    skipped_fallback_missing: Option<(String, Vec<ModelCapability>)>,
    terminal_error: Option<ModelError>,
    done: bool,
}

impl StreamingInvocationState {
    fn new(
        runtime: ConfiguredModelRuntime,
        registry: ModelProviderRegistry,
        requirements: ModelRequestRequirements,
        request_template: ModelRequest,
        attempts: Arc<Mutex<Vec<ModelInvocationAttempt>>>,
    ) -> Self {
        let use_model_override = registry
            .get(&runtime.provider_id)
            .map(|provider| runtime.model != provider.config.default_model)
            .unwrap_or(true);
        attempts.lock().expect("attempts lock").clear();

        Self {
            runtime,
            registry,
            requirements,
            request_template,
            attempts,
            selected_attempts_started: 0,
            fallback_index: 0,
            next_attempt_index: 1,
            use_model_override,
            current_provider_id: None,
            current_model: None,
            current_attempt_position: None,
            current_attempt_started_at: None,
            current_stream: None,
            crossed_side_effect_boundary: false,
            last_failure: None,
            skipped_fallback_missing: None,
            terminal_error: None,
            done: false,
        }
    }

    fn attempts_snapshot(&self) -> Vec<ModelInvocationAttempt> {
        self.attempts.lock().expect("attempts lock").clone()
    }

    fn push_attempt(&mut self, provider_id: &str, model: &str) -> usize {
        let attempt = attempt(provider_id, model, self.next_attempt_index);
        self.next_attempt_index += 1;
        let mut attempts = self.attempts.lock().expect("attempts lock");
        attempts.push(attempt);
        attempts.len().saturating_sub(1)
    }

    fn update_current_attempt_request_id(&self, request_id: &str) {
        let Some(position) = self.current_attempt_position else {
            return;
        };
        if let Some(attempt) = self
            .attempts
            .lock()
            .expect("attempts lock")
            .get_mut(position)
        {
            attempt.request_id = Some(request_id.to_string());
        }
    }

    fn finish_current_attempt(&mut self) {
        let Some(position) = self.current_attempt_position.take() else {
            self.current_attempt_started_at = None;
            return;
        };
        let Some(started_at) = self.current_attempt_started_at.take() else {
            return;
        };
        if let Some(attempt) = self
            .attempts
            .lock()
            .expect("attempts lock")
            .get_mut(position)
        {
            attempt.elapsed_ms = Some(elapsed_ms(started_at.elapsed()));
        }
    }

    fn selected_retry_available(&self, failure: &InvocationFailure) -> bool {
        self.current_provider_id.as_deref() == Some(self.runtime.provider_id.as_str())
            && failure.retryable
            && self
                .runtime
                .invocation_policy
                .retry
                .is_retryable(failure.failure_kind)
            && self.selected_attempts_started < self.runtime.invocation_policy.retry.max_attempts
    }

    fn fallback_available(&self) -> bool {
        self.fallback_index < self.runtime.invocation_policy.fallback.provider_ids.len()
    }

    fn next_candidate(
        &mut self,
    ) -> Result<Option<StreamingInvocationCandidate>, ModelInvocationError> {
        if self.selected_attempts_started < self.runtime.invocation_policy.retry.max_attempts {
            self.selected_attempts_started += 1;
            let provider_id = self.runtime.provider_id.clone();
            let model = self.runtime.model.clone();
            let attempt_position = self.push_attempt(&provider_id, &model);
            return Ok(Some(StreamingInvocationCandidate {
                provider_id,
                model,
                provider: Arc::clone(&self.runtime.provider),
                attempt_position,
            }));
        }

        while self.fallback_index < self.runtime.invocation_policy.fallback.provider_ids.len() {
            let fallback_provider_id =
                self.runtime.invocation_policy.fallback.provider_ids[self.fallback_index].clone();
            self.fallback_index += 1;
            let registered = match self
                .registry
                .require_capabilities(&fallback_provider_id, &self.requirements)
            {
                Ok(registered) => registered,
                Err(ModelRegistryError::ProviderNotFound { id }) => {
                    return Err(ModelInvocationError::FallbackProviderNotRegistered {
                        provider_id: id,
                        attempts: self.attempts_snapshot(),
                    });
                }
                Err(ModelRegistryError::MissingCapabilities {
                    provider_id,
                    missing,
                }) => {
                    self.skipped_fallback_missing
                        .get_or_insert((provider_id, missing));
                    continue;
                }
                Err(error) => {
                    return Err(ModelInvocationError::SelectedProviderUnavailable {
                        provider_id: fallback_provider_id,
                        message: error.to_string(),
                        attempts: self.attempts_snapshot(),
                    });
                }
            };
            let model = if self.use_model_override {
                self.runtime.model.clone()
            } else {
                registered.config.default_model.clone()
            };
            let provider = registered.provider();
            let attempt_position = self.push_attempt(&fallback_provider_id, &model);
            return Ok(Some(StreamingInvocationCandidate {
                provider_id: fallback_provider_id,
                model,
                provider,
                attempt_position,
            }));
        }

        if let Some((provider_id, missing)) = self.skipped_fallback_missing.take() {
            return Err(ModelInvocationError::FallbackMissingCapabilities {
                provider_id,
                missing,
                attempts: self.attempts_snapshot(),
            });
        }

        Ok(None)
    }

    fn handle_failure(&mut self, error: ModelError) {
        let provider_id = self
            .current_provider_id
            .clone()
            .unwrap_or_else(|| self.runtime.provider_id.clone());
        let model = self
            .current_model
            .clone()
            .unwrap_or_else(|| self.runtime.model.clone());
        let failure = InvocationFailure::new(provider_id, model, error);
        let retry_selected = self.selected_retry_available(&failure);
        self.last_failure = Some(failure);
        self.finish_current_attempt();
        self.current_stream = None;
        self.current_provider_id = None;
        self.current_model = None;
        self.crossed_side_effect_boundary = false;

        if retry_selected {
            return;
        }

        self.selected_attempts_started = self.runtime.invocation_policy.retry.max_attempts;
        if !self.fallback_available() {
            self.terminal_error = Some(invocation_error_to_model_error(
                self.last_failure_error()
                    .expect("failure was recorded before terminal error"),
            ));
        }
    }

    fn last_failure_error(&self) -> Option<ModelInvocationError> {
        let failure = self.last_failure.as_ref()?;
        Some(ModelInvocationError::InvocationFailed {
            provider_id: failure.provider_id.clone(),
            model: failure.model.clone(),
            failure_kind: failure.failure_kind,
            retryable: failure.retryable,
            message: failure.message.clone(),
            attempts: self.attempts_snapshot(),
        })
    }
}

struct StreamingInvocationCandidate {
    provider_id: String,
    model: String,
    provider: Arc<dyn ModelProvider>,
    attempt_position: usize,
}

async fn next_streaming_invocation_item(
    mut state: StreamingInvocationState,
) -> Option<(
    Result<ModelStreamEvent, ModelError>,
    StreamingInvocationState,
)> {
    loop {
        if state.done {
            return None;
        }

        if let Some(error) = state.terminal_error.take() {
            state.done = true;
            return Some((Err(error), state));
        }

        if state.current_stream.is_none() {
            let candidate = match state.next_candidate() {
                Ok(Some(candidate)) => candidate,
                Ok(None) => {
                    if let Some(error) = state.last_failure_error() {
                        state.terminal_error = Some(invocation_error_to_model_error(error));
                        continue;
                    }
                    state.done = true;
                    return None;
                }
                Err(error) => {
                    state.terminal_error = Some(invocation_error_to_model_error(error));
                    continue;
                }
            };
            let mut request = state.request_template.clone();
            request.provider = candidate.provider_id.clone();
            request.model = candidate.model.clone();
            state.current_provider_id = Some(candidate.provider_id);
            state.current_model = Some(candidate.model);
            state.current_attempt_position = Some(candidate.attempt_position);
            state.current_attempt_started_at = Some(Instant::now());
            state.crossed_side_effect_boundary = false;

            match candidate.provider.stream(request).await {
                Ok(stream) => state.current_stream = Some(stream),
                Err(error) => state.handle_failure(error),
            }
            continue;
        }

        let stream = state
            .current_stream
            .as_mut()
            .expect("current stream checked above");
        match stream.next().await {
            Some(Ok(event)) => {
                if let ModelStreamEvent::Started { request_id } = &event {
                    state.update_current_attempt_request_id(request_id);
                }
                if model_event_crosses_side_effect_boundary(&event) {
                    state.crossed_side_effect_boundary = true;
                }
                return Some((Ok(event), state));
            }
            Some(Err(error)) => {
                state.finish_current_attempt();
                if state.crossed_side_effect_boundary {
                    state.done = true;
                    return Some((Err(error), state));
                }
                state.handle_failure(error);
            }
            None => {
                state.finish_current_attempt();
                state.done = true;
                return None;
            }
        }
    }
}

fn model_event_crosses_side_effect_boundary(event: &ModelStreamEvent) -> bool {
    matches!(
        event,
        ModelStreamEvent::ReasoningSummaryDelta { .. }
            | ModelStreamEvent::Delta { .. }
            | ModelStreamEvent::ToolCall { .. }
    )
}

fn invocation_error_to_model_error(error: ModelInvocationError) -> ModelError {
    ModelError::Provider {
        code: "model.invocation_policy.failed".to_string(),
        message: error.to_string(),
        retryable: false,
    }
}

async fn invoke(
    provider: Arc<dyn ModelProvider>,
    provider_id: &str,
    model: &str,
    request_template: &ModelRequest,
) -> (Result<ModelStreamResult, ModelError>, Option<String>) {
    let mut request = request_template.clone();
    request.provider = provider_id.to_string();
    request.model = model.to_string();
    let stream = match provider.stream(request).await {
        Ok(stream) => stream,
        Err(error) => return (Err(error), None),
    };
    collect_model_stream_with_request_id(stream).await
}

async fn collect_model_stream_with_request_id(
    mut stream: ruse_model::ModelEventStream,
) -> (Result<ModelStreamResult, ModelError>, Option<String>) {
    let mut events = Vec::new();
    let mut request_id = None;
    while let Some(item) = stream.next().await {
        match item {
            Ok(event) => {
                if request_id.is_none()
                    && let ModelStreamEvent::Started { request_id: id } = &event
                {
                    request_id = Some(id.clone());
                }
                events.push(event);
            }
            Err(error) => return (Err(error), request_id),
        }
    }
    (Ok(events), request_id)
}

fn complete_attempt(
    attempt: &mut ModelInvocationAttempt,
    duration: Duration,
    request_id: Option<String>,
) {
    attempt.elapsed_ms = Some(elapsed_ms(duration));
    attempt.request_id = request_id;
}

fn elapsed_ms(duration: Duration) -> u64 {
    duration.as_millis().try_into().unwrap_or(u64::MAX)
}

fn attempt(provider_id: &str, model: &str, attempt_index: u32) -> ModelInvocationAttempt {
    ModelInvocationAttempt {
        provider_id: provider_id.to_string(),
        model: model.to_string(),
        attempt_index,
        request_id: None,
        elapsed_ms: None,
    }
}

struct InvocationFailure {
    provider_id: String,
    model: String,
    failure_kind: ModelFailureKind,
    retryable: bool,
    message: String,
}

impl InvocationFailure {
    fn new(provider_id: String, model: String, error: ModelError) -> Self {
        let (failure_kind, retryable) = classify_model_error(&error);
        Self {
            provider_id,
            model,
            failure_kind,
            retryable,
            message: error.to_string(),
        }
    }
}

fn classify_model_error(error: &ModelError) -> (ModelFailureKind, bool) {
    match error {
        ModelError::Script(_) => (ModelFailureKind::Unknown, false),
        ModelError::Provider {
            code, retryable, ..
        } => (classify_provider_error_code(code), *retryable),
    }
}

fn classify_provider_error_code(code: &str) -> ModelFailureKind {
    let code = code.to_ascii_lowercase();
    if code.contains("timeout") {
        ModelFailureKind::Timeout
    } else if code.contains("rate") {
        ModelFailureKind::RateLimited
    } else if code.contains("transport") || code.contains("network") || code.contains("connection")
    {
        ModelFailureKind::Transport
    } else if code.contains("credential")
        || code.contains("auth")
        || code.contains("401")
        || code.contains("unauthorized")
    {
        ModelFailureKind::InvalidCredentials
    } else if code.contains("unsupported") {
        ModelFailureKind::UnsupportedRequest
    } else if code.contains("reject") || code.contains("provider_response") {
        ModelFailureKind::ProviderRejected
    } else {
        ModelFailureKind::Unknown
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum ModelRuntimeError {
    #[error("invalid model runtime selection at {field}: {message}")]
    InvalidSelection { field: String, message: String },
    #[error("model provider is disabled: {provider_id}")]
    ProviderDisabled { provider_id: String },
    #[error("model provider is not registered: {provider_id}")]
    ProviderNotRegistered { provider_id: String },
    #[error("model provider {provider_id} is missing capabilities: {missing:?}")]
    MissingCapabilities {
        provider_id: String,
        missing: Vec<ModelCapability>,
    },
}

impl ModelRuntimeError {
    fn from_registry(error: ModelRegistryError) -> Self {
        match error {
            ModelRegistryError::ProviderNotFound { id } => {
                Self::ProviderNotRegistered { provider_id: id }
            }
            ModelRegistryError::MissingCapabilities {
                provider_id,
                missing,
            } => Self::MissingCapabilities {
                provider_id,
                missing,
            },
            ModelRegistryError::DuplicateProviderId { id } => Self::InvalidSelection {
                field: "model.providers".to_string(),
                message: format!("duplicate registered provider id: {id}"),
            },
            ModelRegistryError::InvalidProviderConfig(error) => Self::InvalidSelection {
                field: "model.providers".to_string(),
                message: format!("invalid registered provider config: {error}"),
            },
            ModelRegistryError::InvalidCapabilities {
                provider_id,
                source,
            } => Self::InvalidSelection {
                field: format!("model.providers.{provider_id}.capabilities"),
                message: format!("invalid registered provider capabilities: {source}"),
            },
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum ModelInvocationError {
    #[error("selected model provider {provider_id} is unavailable: {message}")]
    SelectedProviderUnavailable {
        provider_id: String,
        message: String,
        attempts: Vec<ModelInvocationAttempt>,
    },
    #[error("fallback model provider is not registered: {provider_id}")]
    FallbackProviderNotRegistered {
        provider_id: String,
        attempts: Vec<ModelInvocationAttempt>,
    },
    #[error("fallback model provider {provider_id} is missing capabilities: {missing:?}")]
    FallbackMissingCapabilities {
        provider_id: String,
        missing: Vec<ModelCapability>,
        attempts: Vec<ModelInvocationAttempt>,
    },
    #[error("model invocation failed for {provider_id}/{model}: {message}")]
    InvocationFailed {
        provider_id: String,
        model: String,
        failure_kind: ModelFailureKind,
        retryable: bool,
        message: String,
        attempts: Vec<ModelInvocationAttempt>,
    },
}

impl ModelInvocationError {
    #[must_use]
    pub fn attempts(&self) -> &[ModelInvocationAttempt] {
        match self {
            Self::SelectedProviderUnavailable { attempts, .. }
            | Self::FallbackProviderNotRegistered { attempts, .. }
            | Self::FallbackMissingCapabilities { attempts, .. }
            | Self::InvocationFailed { attempts, .. } => attempts,
        }
    }

    fn selected_provider_unavailable(error: ModelRegistryError) -> Self {
        match error {
            ModelRegistryError::ProviderNotFound { id } => Self::SelectedProviderUnavailable {
                provider_id: id,
                message: "selected provider was not found in the registry".to_string(),
                attempts: Vec::new(),
            },
            error => Self::SelectedProviderUnavailable {
                provider_id: "selected".to_string(),
                message: error.to_string(),
                attempts: Vec::new(),
            },
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ConfiguredTurnError {
    #[error("invalid configured turn input at {field}: {message}")]
    InvalidInput { field: String, message: String },
    #[error(transparent)]
    Runtime(#[from] ModelRuntimeError),
    #[error("configured turn model failed: {message}")]
    Model {
        message: String,
        events: Vec<RuntimeEvent>,
        attempts: Vec<ModelInvocationAttempt>,
    },
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ConfiguredTurnPersistenceError {
    #[error(transparent)]
    Turn(#[from] ConfiguredTurnError),
    #[error("configured turn event persistence failed: {message}")]
    Storage { message: String },
}
