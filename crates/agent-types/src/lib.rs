#![forbid(unsafe_code)]
//! Shared types for the agent.

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
