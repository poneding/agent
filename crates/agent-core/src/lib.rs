#![forbid(unsafe_code)]
//! Core runtime orchestration for agent.

pub mod context;
pub mod runtime;
pub mod tool_loop;
pub mod turn;

pub use context::ContextAssembler;
pub use runtime::RuntimeIds;
pub use tool_loop::{ToolCallRequest, ToolLoop, ToolLoopError, ToolLoopOutcome, ToolLoopStatus};
pub use turn::{TurnEngine, TurnError, TurnInput, TurnOutcome};

/// Returns a smoke-test banner proving the core crate links model and storage.
#[must_use]
pub fn runtime_banner() -> String {
    format!(
        "agent ({}, {})",
        agent_model::model_layer_name(),
        agent_storage::storage_layer_name()
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_runtime_banner() {
        assert_eq!(
            crate::runtime_banner(),
            "agent (agent-model uses agent-types, agent-storage uses agent-types)"
        );
    }
}
