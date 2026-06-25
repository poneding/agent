#![forbid(unsafe_code)]
//! Durable storage contracts for agent.

pub mod checkpoint;
pub mod event_store;
pub mod memory;
pub mod replay;
pub mod sqlite;

pub use checkpoint::{CheckpointRecord, CheckpointStore};
pub use event_store::{EventStore, StorageError, StoredEvent};
pub use memory::InMemoryEventStore;
pub use replay::{ReplayCursor, ReplayEvent};
pub use sqlite::SqliteEventStore;

/// Returns a smoke-test string proving the storage crate links shared types.
#[must_use]
pub fn storage_layer_name() -> String {
    format!("agent-storage uses {}", agent_types::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_storage_layer_identity() {
        assert_eq!(
            crate::storage_layer_name(),
            "agent-storage uses agent-types"
        );
    }
}
