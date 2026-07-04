#![forbid(unsafe_code)]
//! Worktree isolation contracts for agent.

pub mod conflict;
pub mod error;
pub mod lease;
pub mod manager;

pub use conflict::{
    ChangeKind, Conflict, ConflictKind, ConflictReport, FileChange, detect_conflicts,
};
pub use error::WorktreeError;
pub use lease::{
    WorktreeCleanupPolicy, WorktreeLease, WorktreeLeaseId, WorktreeLeaseState, WorktreeRequest,
};
pub use manager::{InMemoryWorktreeManager, WorktreeManager};
