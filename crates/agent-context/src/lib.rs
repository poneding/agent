#![forbid(unsafe_code)]
//! Context assembly contracts for agent.

pub mod error;
pub mod prompt;

pub use error::ContextError;
pub use prompt::{
    PromptBundle, PromptLayer, PromptLayerKind, PromptMessage, assemble_prompt_bundle,
};

/// Returns a smoke-test string proving the context crate links shared types.
#[must_use]
pub fn context_layer_name() -> String {
    format!("agent-context uses {}", agent_types::crate_name())
}
