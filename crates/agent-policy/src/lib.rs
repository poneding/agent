#![forbid(unsafe_code)]
//! Permission policy contracts for agent.

pub mod approval;
pub mod capability;
pub mod decision;
pub mod engine;
pub mod error;
pub mod lease;
pub mod request;

pub use approval::{
    ApprovalDecision, ApprovalLeaseGrant, ApprovalLeaseScope, ApprovalPreviewSummary,
    ApprovalResponse, ApprovalResumeToken, ApprovalStatus, PendingApproval,
};
pub use capability::{CapabilityAction, ResourceKind, ResourceScope};
pub use decision::PolicyDecision;
pub use engine::{PolicyEngine, StaticPolicyEngine};
pub use error::PolicyError;
pub use lease::PermissionLease;
pub use request::PermissionRequest;
