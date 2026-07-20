#![forbid(unsafe_code)]
//! Durable storage contracts for agent.

pub mod checkpoint;
pub mod compatibility;
pub mod event_store;
pub mod idempotency;
pub mod memory;
pub mod migration;
pub mod replay;
pub mod sqlite;

pub use checkpoint::{CheckpointRecord, CheckpointStore};
pub use compatibility::{
    ReplayCompatibilityError, ReplayCompatibilityExpected, ReplayCompatibilityFixture,
    ReplayCompatibilityReport, check_replay_compatibility, load_replay_fixture_json,
};
pub use event_store::{EventStore, StorageError, StoredEvent};
pub use idempotency::{IdempotencyKey, IdempotencyStore, ToolEffectRecord};
pub use memory::InMemoryEventStore;
pub use migration::{
    MigrationDiagnostic, MigrationDiagnosticSeverity, MigrationError, MigrationOutcome,
    MigrationRegistry, MigrationReport, SchemaFamily, SchemaMigration, SchemaVersion,
};
pub use replay::{ReplayCursor, ReplayEvent};
pub use sqlite::SqliteEventStore;

/// Returns a smoke-test string proving the storage crate links shared types.
#[must_use]
pub fn storage_layer_name() -> String {
    format!("ruse-storage uses {}", ruse_types::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_storage_layer_identity() {
        assert_eq!(
            crate::storage_layer_name(),
            "ruse-storage uses ruse-types"
        );
    }
}
