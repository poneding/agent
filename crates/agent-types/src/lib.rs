#![forbid(unsafe_code)]
//! Shared durable contracts for agent.

pub mod actor;
pub mod error;
pub mod event;
pub mod external;
pub mod ids;
pub mod plan;
pub mod schema;
pub mod state;
pub mod task;
pub mod timestamp;
pub mod tool;

pub use actor::{ActorKind, ActorRef};
pub use error::{AgentError, AgentTypesError, RuntimeError, RuntimeWarning};
pub use event::*;
pub use external::*;
pub use ids::*;
pub use plan::*;
pub use schema::SchemaVersion;
pub use state::{TaskState, TurnState};
pub use task::*;
pub use timestamp::Timestamp;
pub use tool::*;

/// Returns this crate's stable package name.
#[must_use]
pub const fn crate_name() -> &'static str {
    "agent-types"
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_identity() {
        assert_eq!(crate::crate_name(), "agent-types");
    }
}
