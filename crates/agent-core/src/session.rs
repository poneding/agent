//! Thin session facade over the multi-layer turn runners.
//!
//! Product surfaces (CLI/TUI) should prefer this entrypoint over constructing
//! `ConfiguredTurnInput` / `ComprehensiveTurnRunner` pieces by hand. The
//! underlying runners remain public for tests and specialized hosts.

use std::sync::Arc;

use agent_context::PromptMessage;
use agent_model::{ModelProviderRegistry, ModelRequestRequirements, ModelToolDefinition};
use agent_policy::PolicyEngine;
use agent_tool::ToolRegistry;
use agent_types::{ReasoningEffort, RuntimeEvent};

use crate::config::EffectiveRuntimeConfig;
use crate::model_runtime::{
    AgentLoopBudget, AgentLoopOutcome, ComprehensiveTurnInput, ComprehensiveTurnResources,
    ComprehensiveTurnRunner, ComprehensiveTurnStreamOptions, ConfiguredTurnError,
    ConfiguredTurnInput, ConfiguredTurnOutcome, ConfiguredTurnRunner, ConfiguredTurnStream,
    ConfiguredTurnStreamOptions, PendingComprehensiveApproval,
};
use crate::runtime::RuntimeIds;
use crate::turn::CancellationToken;

/// One interactive agent session: ids + config + model requirements.
///
/// Cheap to clone; bulky config/history are shared via `Arc` inside turn inputs.
#[derive(Clone, Debug)]
pub struct AgentSession {
    pub ids: RuntimeIds,
    pub effective_config: Arc<EffectiveRuntimeConfig>,
    pub requirements: ModelRequestRequirements,
    pub loop_budget: AgentLoopBudget,
}

impl AgentSession {
    #[must_use]
    pub fn new(
        ids: RuntimeIds,
        effective_config: EffectiveRuntimeConfig,
        requirements: ModelRequestRequirements,
        loop_budget: AgentLoopBudget,
    ) -> Self {
        Self {
            ids,
            effective_config: Arc::new(effective_config),
            requirements,
            loop_budget,
        }
    }

    #[must_use]
    pub fn with_shared_config(
        ids: RuntimeIds,
        effective_config: Arc<EffectiveRuntimeConfig>,
        requirements: ModelRequestRequirements,
        loop_budget: AgentLoopBudget,
    ) -> Self {
        Self {
            ids,
            effective_config,
            requirements,
            loop_budget,
        }
    }

    /// Model-only turn (no tools). Used by batch / capability fallback paths.
    pub async fn prompt(
        &self,
        user_message: impl Into<String>,
        conversation_history: Vec<PromptMessage>,
        registry: &ModelProviderRegistry,
    ) -> Result<ConfiguredTurnOutcome, ConfiguredTurnError> {
        ConfiguredTurnRunner::run(
            ConfiguredTurnInput::from_shared(
                self.ids,
                user_message,
                Arc::new(conversation_history),
                Arc::clone(&self.effective_config),
                self.requirements.clone(),
            ),
            registry,
        )
        .await
    }

    /// Streaming model-only turn.
    pub async fn prompt_stream(
        &self,
        user_message: impl Into<String>,
        conversation_history: Vec<PromptMessage>,
        registry: &ModelProviderRegistry,
        options: SessionPromptOptions,
    ) -> Result<ConfiguredTurnStream, ConfiguredTurnError> {
        ConfiguredTurnRunner::run_stream_with_options(
            ConfiguredTurnInput::from_shared(
                self.ids,
                user_message,
                Arc::new(conversation_history),
                Arc::clone(&self.effective_config),
                self.requirements.clone(),
            ),
            registry,
            ConfiguredTurnStreamOptions {
                coding_context: false,
                cancellation: options.cancellation,
                tools: options.tools,
                reasoning_effort: options.reasoning_effort,
            },
        )
        .await
    }

    /// Full agent loop with tools, policy, budgets, and optional live sinks.
    pub async fn prompt_with_tools<P, S, A>(
        &self,
        user_message: impl Into<String>,
        conversation_history: Vec<PromptMessage>,
        registry: &ModelProviderRegistry,
        tools: &ToolRegistry,
        policy: &P,
        options: SessionAgentOptions,
        mut event_sink: S,
        mut approval_sink: A,
    ) -> Result<AgentLoopOutcome, ConfiguredTurnError>
    where
        P: PolicyEngine + ?Sized,
        S: FnMut(RuntimeEvent) -> Result<(), ConfiguredTurnError>,
        A: FnMut(PendingComprehensiveApproval) -> Result<(), ConfiguredTurnError>,
    {
        let input = ComprehensiveTurnInput {
            ids: self.ids,
            user_message: user_message.into(),
            conversation_history: Arc::new(conversation_history),
            effective_config: Arc::clone(&self.effective_config),
            requirements: self.requirements.clone(),
            loop_budget: options.loop_budget.unwrap_or(self.loop_budget.clone()),
        };
        ComprehensiveTurnRunner::run_with_resources_streaming_and_approval_sink(
            input,
            registry,
            ComprehensiveTurnResources { tools, policy },
            ComprehensiveTurnStreamOptions {
                coding_context: options.coding_context,
                cancellation: options.cancellation,
                reasoning_effort: options.reasoning_effort,
            },
            &mut event_sink,
            &mut approval_sink,
        )
        .await
    }
}

#[derive(Clone, Debug, Default)]
pub struct SessionPromptOptions {
    pub cancellation: CancellationToken,
    pub tools: Arc<Vec<ModelToolDefinition>>,
    pub reasoning_effort: Option<ReasoningEffort>,
}

#[derive(Clone, Debug, Default)]
pub struct SessionAgentOptions {
    pub cancellation: CancellationToken,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub loop_budget: Option<AgentLoopBudget>,
    /// Load AGENTS.md/CLAUDE.md + git status into the model prompt.
    pub coding_context: bool,
}


