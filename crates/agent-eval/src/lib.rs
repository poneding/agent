#![forbid(unsafe_code)]
//! Evaluation harness contracts for runtime regression scenarios.

pub mod golden;
pub mod manifest;
pub mod report;
pub mod runner;

pub use golden::{
    GoldenComparisonError, GoldenComparisonOptions, GoldenComparisonResult, GoldenMismatch,
    compare_report_to_golden_file, compare_report_to_golden_json, normalized_report_golden,
};
pub use manifest::{
    EVAL_SCENARIO_SCHEMA_VERSION, EvalDiagnostic, EvalDiagnosticSeverity, EvalExpectations,
    EvalExpectedStatus, EvalFixtures, EvalManifestError, EvalPermissionProfile, EvalScenarioKind,
    EvalScenarioManifest, ValidatedEvalFixtures, ValidatedEvalScenario, parse_eval_scenario_json,
    validate_eval_scenario,
};
pub use report::{
    EVAL_RUN_REPORT_SCHEMA_VERSION, EvalEvidenceSource, EvalEvidenceSummary, EvalRunReport,
    EvalRunSummary, EvalScenarioReport, EvalScenarioStatus,
};
pub use runner::{EvalRunnerOptions, EvalStrictness, run_eval_scenario, run_eval_scenarios};

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
