#![forbid(unsafe_code)]
//! Worktree isolation contracts for agent.

pub mod error;
pub mod lease;
pub mod manager;

pub use error::WorktreeError;
pub use lease::{
    WorktreeCleanupPolicy, WorktreeLease, WorktreeLeaseId, WorktreeLeaseState, WorktreeRequest,
};
pub use manager::{InMemoryWorktreeManager, WorktreeManager};
