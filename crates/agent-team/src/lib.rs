#![forbid(unsafe_code)]
//! Team and subagent delegation contracts for agent.

pub mod capability;
pub mod error;
pub mod handoff;

pub use capability::{CapabilityGrant, CapabilitySet, reduce_capabilities};
pub use error::TeamError;
pub use handoff::{
    HandoffEnvelope, HandoffStatus, SubagentProfile, SubagentRole, transition_handoff_status,
};
