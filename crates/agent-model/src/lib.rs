#![forbid(unsafe_code)]
//! Model provider contracts for agent.

pub mod fake;
pub mod provider;
pub mod request;
pub mod stream;

pub use fake::{FakeModelProvider, FakeModelScript};
pub use provider::{ModelError, ModelProvider};
pub use request::ModelRequest;
pub use stream::{ModelStreamEvent, ModelStreamResult};

/// Returns a smoke-test string proving the model crate links shared types.
#[must_use]
pub fn model_layer_name() -> String {
    format!("agent-model uses {}", agent_types::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_model_layer_identity() {
        assert_eq!(crate::model_layer_name(), "agent-model uses agent-types");
    }
}
