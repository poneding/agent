#![forbid(unsafe_code)]
//! Evaluation harness contracts for runtime regression scenarios.

pub mod golden;
pub mod manifest;
pub mod report;
pub mod runner;
pub mod suite;

pub use golden::{
    GoldenComparisonError, GoldenComparisonOptions, GoldenComparisonResult, GoldenMismatch,
    GoldenUpdateError, GoldenUpdateOptions, GoldenUpdateReport, compare_report_to_golden_file,
    compare_report_to_golden_json, normalized_report_golden, update_suite_goldens,
};
pub use manifest::{
    EVAL_SCENARIO_SCHEMA_VERSION, EvalContextQualityExpectations, EvalDiagnostic,
    EvalDiagnosticSeverity, EvalExpectations, EvalExpectedStatus, EvalFixtures, EvalManifestError,
    EvalMemoryExpectations, EvalPermissionProfile, EvalRecoveryExpectations, EvalScenarioKind,
    EvalScenarioManifest, EvalTeamWorkflowExpectations, ValidatedEvalFixtures,
    ValidatedEvalScenario, parse_eval_scenario_json, validate_eval_scenario,
};
pub use report::{
    EVAL_RUN_REPORT_SCHEMA_VERSION, EvalArtifactOptions, EvalArtifactSet, EvalEvidenceSource,
    EvalEvidenceSummary, EvalReportArtifactError, EvalRunReport, EvalRunSummary,
    EvalScenarioReport, EvalScenarioStatus, render_eval_summary_text, write_eval_artifacts,
};
pub use runner::{EvalRunnerOptions, EvalStrictness, run_eval_scenario, run_eval_scenarios};
pub use suite::{
    EVAL_SUITE_SCHEMA_VERSION, EvalSuiteError, EvalSuiteManifest, EvalSuiteRunnerOptions,
    EvalSuiteScenarioRef, ValidatedEvalSuite, ValidatedEvalSuiteScenarioRef, parse_eval_suite_json,
    run_eval_suite, validate_eval_suite,
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
