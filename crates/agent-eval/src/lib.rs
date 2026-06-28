#![forbid(unsafe_code)]
//! Evaluation harness contracts for runtime regression scenarios.

pub mod manifest;

pub use manifest::{
    EVAL_SCENARIO_SCHEMA_VERSION, EvalDiagnostic, EvalDiagnosticSeverity, EvalExpectations,
    EvalExpectedStatus, EvalFixtures, EvalManifestError, EvalPermissionProfile, EvalScenarioKind,
    EvalScenarioManifest, ValidatedEvalFixtures, ValidatedEvalScenario, parse_eval_scenario_json,
    validate_eval_scenario,
};

/// Returns this crate's stable package name.
#[must_use]
pub const fn crate_name() -> &'static str {
    "agent-eval"
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_identity() {
        assert_eq!(crate::crate_name(), "agent-eval");
    }
}
