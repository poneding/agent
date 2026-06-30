#![forbid(unsafe_code)]
//! Context assembly contracts for agent.

pub mod budget;
pub mod compact;
pub mod error;
pub mod memory_layers;
pub mod project;
pub mod prompt;
pub mod retrieval;
pub mod semantic;

pub use budget::{
    BudgetClass, BudgetDecision, BudgetItem, BudgetPlan, ContextBudget, plan_context_budget,
};
pub use compact::{
    CompactRequest, CompactResult, CompactionSummary, PinnedMessage, compact_messages,
};
pub use error::ContextError;
pub use memory_layers::{memory_prompt_layer, memory_prompt_layers};
pub use project::{
    ProjectContextDiagnostic, ProjectContextDiagnosticSeverity, ProjectContextLimits,
    ProjectContextOmission, ProjectContextSummary, ProjectContextVisibility,
    ProjectDirectorySummary, ProjectFileKind, ProjectFileSummary,
};
pub use prompt::{
    PromptAssemblyOutcome, PromptAssemblyReport, PromptAssemblyRequest, PromptBudget, PromptBundle,
    PromptDiagnostic, PromptDiagnosticSeverity, PromptLayer, PromptLayerHash, PromptLayerId,
    PromptLayerKind, PromptLayerPolicy, PromptLayerSensitivity, PromptLayerSource,
    PromptLayerVisibility, PromptMessage, PromptOverlay, PromptOverlayAction, PromptProfile,
    PromptProfileId, PromptRuntime, PromptSnapshot, assemble_prompt_bundle,
};
pub use retrieval::{
    ContextCandidate, ContextCandidateOmission, ContextCandidateOmissionReason,
    ContextRetrievalDiagnostic, ContextRetrievalDiagnosticSeverity, ContextRetrievalQuery,
    ContextRetrievalReport, ContextSelectionReason, retrieve_relevant_context,
};
pub use semantic::{
    ContextHybridCandidate, ContextHybridCandidateOmission, ContextHybridRetrievalReport,
    ContextSemanticFallbackPolicy, ContextSemanticRetrievalConfig, ContextSemanticRetrievalMode,
    ContextSemanticRetrievalStatus, ContextSemanticRetrievalSummary, ProjectSemanticDocument,
    build_project_semantic_documents, retrieve_hybrid_context,
};

/// Returns a smoke-test string proving the context crate links shared types.
#[must_use]
pub fn context_layer_name() -> String {
    format!("agent-context uses {}", agent_types::crate_name())
}
