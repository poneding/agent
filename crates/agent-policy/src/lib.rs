#![forbid(unsafe_code)]
//! Permission policy contracts for agent.

pub mod capability;
pub mod decision;
pub mod engine;
pub mod error;
pub mod lease;
pub mod request;

pub use capability::{CapabilityAction, ResourceKind, ResourceScope};
pub use decision::PolicyDecision;
pub use engine::{PolicyEngine, StaticPolicyEngine};
pub use error::PolicyError;
pub use lease::PermissionLease;
pub use request::PermissionRequest;
