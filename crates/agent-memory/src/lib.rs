#![forbid(unsafe_code)]
//! Scoped memory contracts for agent.

pub mod archive;
pub mod audit;
pub mod embedding;
pub mod error;
pub mod index;
pub mod record;
pub mod store;

pub use archive::{
    MEMORY_ARCHIVE_SCHEMA_VERSION, MemoryArchive, MemoryArchiveError, MemoryArchiveManifest,
    MemoryArchiveRecord, MemoryImportOutcome, MemoryImportReport, MemoryRetentionPolicy,
    export_memory_archive, import_memory_archive,
};
pub use audit::{AuditedMemoryStore, MemoryAuditAction, MemoryAuditEntry};
pub use embedding::{
    DeterministicEmbeddingProvider, EmbeddingBatch, EmbeddingInput, EmbeddingMetadata,
    EmbeddingProvider, EmbeddingRequest, EmbeddingVector,
};
pub use error::MemoryError;
pub use index::{
    HybridMemoryQuery, InMemoryMemoryIndex, MemoryIndex, MemoryIndexStats, MemoryRetrievalCandidate,
};
pub use record::{MemoryRecord, MemoryScope, MemorySensitivity, MemorySource};
pub use store::{InMemoryMemoryStore, MemoryStore};

/// Returns a smoke-test string proving the memory crate links shared types.
#[must_use]
pub fn memory_layer_name() -> String {
    format!("agent-memory uses {}", agent_types::crate_name())
}
