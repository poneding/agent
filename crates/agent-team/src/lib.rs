#![forbid(unsafe_code)]
//! Team and subagent delegation contracts for agent.

pub mod blackboard;
pub mod capability;
pub mod error;
pub mod handoff;
pub mod merge_review;
pub mod workflow;

pub use blackboard::{
    ArtifactRef, BlackboardEntry, BlackboardEntryKind, BlackboardStore, InMemoryBlackboardStore,
    ReviewVote,
};
pub use capability::{CapabilityGrant, CapabilitySet, reduce_capabilities};
pub use error::TeamError;
pub use handoff::{
    HandoffEnvelope, HandoffStatus, SubagentProfile, SubagentRole, transition_handoff_status,
};
pub use merge_review::{TeamMergeBlocker, TeamMergeBlockerKind, TeamMergeReadinessReport};
pub use workflow::{
    TeamBlackboardEntrySummary, TeamBlackboardSummary, TeamCapabilityCeilingSummary,
    TeamCapabilityGrantSummary, TeamHandoffSummary, TeamMergeReviewSummary, TeamWorkflowDiagnostic,
    TeamWorkflowDiagnosticSeverity, TeamWorkflowReadiness, TeamWorkflowSnapshot,
    TeamWorkflowStatus, TeamWorktreeState, TeamWorktreeSummary,
};
