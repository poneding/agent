#![forbid(unsafe_code)]
//! Scoped memory contracts for agent.

pub mod archive;
pub mod audit;
pub mod editor;
pub mod embedding;
pub mod error;
pub mod index;
pub mod proposal;
pub mod record;
pub mod rerank;
pub mod store;
pub mod tombstone;
pub mod trace;

pub use archive::{
    MEMORY_ARCHIVE_SCHEMA_VERSION, MemoryArchive, MemoryArchiveError, MemoryArchiveManifest,
    MemoryArchiveRecord, MemoryImportOutcome, MemoryImportReport, MemoryRetentionPolicy,
    export_memory_archive, import_memory_archive,
};
pub use audit::{AuditedMemoryStore, MemoryAuditAction, MemoryAuditEntry};
pub use editor::{
    InMemoryMemoryEditor, MemoryEditor, MemoryEditorSummary, MemoryImportPolicy,
    MemoryIndexRebuildReport, MemoryTombstoneConflictPolicy,
};
pub use embedding::{
    DeterministicEmbeddingProvider, EmbeddingBatch, EmbeddingInput, EmbeddingMetadata,
    EmbeddingProvider, EmbeddingRequest, EmbeddingVector,
};
pub use error::MemoryError;
pub use index::{
    HybridMemoryQuery, InMemoryMemoryIndex, MemoryIndex, MemoryIndexStats, MemoryRetrievalCandidate,
};
pub use proposal::{
    InMemoryMemoryProposalStore, MemoryAutoApprovalEligibility, MemoryProposalDecision,
    MemoryProposalStatus, MemoryProposalStore, MemoryProposalSummary, MemoryWriteProposal,
};
pub use record::{MemoryRecord, MemoryScope, MemorySensitivity, MemorySource};
pub use rerank::{
    DeterministicMemoryReranker, MemoryRejectedCandidate, MemoryRerankRequest, MemoryReranker,
    MemoryRetrievalReason, MemoryRetrievalResult, MemorySelectedCandidate,
};
pub use store::{InMemoryMemoryStore, MemoryStore};
pub use tombstone::MemoryTombstone;
pub(crate) use trace::stable_trace_hash;
pub use trace::{MemoryInfluenceEntry, MemoryInfluenceTrace};

/// Returns a smoke-test string proving the memory crate links shared types.
#[must_use]
pub fn memory_layer_name() -> String {
    format!("agent-memory uses {}", agent_types::crate_name())
}
