#![forbid(unsafe_code)]
//! Context assembly contracts for agent.

pub mod budget;
pub mod compact;
pub mod error;
pub mod prompt;

pub use budget::{
    BudgetClass, BudgetDecision, BudgetItem, BudgetPlan, ContextBudget, plan_context_budget,
};
pub use compact::{
    CompactRequest, CompactResult, CompactionSummary, PinnedMessage, compact_messages,
};
pub use error::ContextError;
pub use prompt::{
    PromptAssemblyOutcome, PromptAssemblyReport, PromptAssemblyRequest, PromptBudget, PromptBundle,
    PromptDiagnostic, PromptDiagnosticSeverity, PromptLayer, PromptLayerHash, PromptLayerId,
    PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource,
    PromptLayerVisibility, PromptMessage, PromptOverlay, PromptOverlayAction, PromptProfile,
    PromptProfileId, PromptRuntime, PromptSnapshot, assemble_prompt_bundle,
};

/// Returns a smoke-test string proving the context crate links shared types.
#[must_use]
pub fn context_layer_name() -> String {
    format!("agent-context uses {}", agent_types::crate_name())
}
