#![forbid(unsafe_code)]
//! Scoped memory contracts for agent.

pub mod error;
pub mod record;
pub mod store;

pub use error::MemoryError;
pub use record::{MemoryRecord, MemoryScope, MemorySensitivity, MemorySource};
pub use store::{InMemoryMemoryStore, MemoryStore};

/// Returns a smoke-test string proving the memory crate links shared types.
#[must_use]
pub fn memory_layer_name() -> String {
    format!("agent-memory uses {}", agent_types::crate_name())
}
