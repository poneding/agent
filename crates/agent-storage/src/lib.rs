#![forbid(unsafe_code)]
//! Storage abstractions for runtime events, checkpoints, and artifacts.

/// Returns a smoke-test string proving the storage crate links shared types.
#[must_use]
pub fn storage_layer_name() -> String {
    format!("agent-storage uses {}", agent_types::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_storage_layer_identity() {
        assert_eq!(crate::storage_layer_name(), "agent-storage uses agent-types");
    }
}
