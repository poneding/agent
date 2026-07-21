use std::fs;
use std::path::PathBuf;

use camino::Utf8PathBuf;
use ruse_eval::{
    EvalRunnerOptions, EvalScenarioStatus, GoldenComparisonOptions, compare_report_to_golden_file,
    parse_eval_suite_json, run_eval_scenarios,
};

fn eval_root() -> Utf8PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/eval");
    let canonical = fs::canonicalize(root).expect("canonical eval root");
    Utf8PathBuf::from_path_buf(canonical).expect("utf8 eval root")
}

fn load_default_suite() -> ruse_eval::ValidatedEvalSuite {
    let root = eval_root();
    let input = fs::read_to_string(root.join("default.suite.json").as_std_path())
        .expect("read default suite");
    parse_eval_suite_json(&input, &root).expect("default suite should load")
}

#[test]
fn builtin_corpus_loads_default_suite() {
    let suite = load_default_suite();

    assert_eq!(suite.manifest.id, "default-regression");
    assert_eq!(suite.scenario_refs.len(), 7);
    assert_eq!(suite.scenarios.len(), 7);
    assert_eq!(suite.scenario_refs[0].alias.as_deref(), Some("m04-replay"));
    assert_eq!(
        suite.scenario_refs[6].alias.as_deref(),
        Some("m64-memory-smoke")
    );
    assert_eq!(
        suite.scenarios[0].normalized_fixture_root,
        suite.normalized_suite_root
    );
}

#[test]
fn builtin_corpus_runs_and_matches_goldens() {
    let suite = load_default_suite();

    let report = run_eval_scenarios(
        &suite.scenarios,
        EvalRunnerOptions {
            run_id: "builtin-corpus-test".to_string(),
            ..EvalRunnerOptions::default()
        },
    );

    assert_eq!(report.summary.total, 7);
    assert_eq!(report.summary.passed, 7);
    assert!(report.scenarios.iter().all(|scenario| {
        scenario.status == EvalScenarioStatus::Passed && scenario.diagnostics.is_empty()
    }));

    for (scenario, scenario_report) in suite.scenarios.iter().zip(report.scenarios.iter()) {
        let golden_path = scenario
            .normalized_fixtures
            .golden
            .as_ref()
            .expect("scenario golden path");
        let comparison = compare_report_to_golden_file(
            scenario_report,
            golden_path,
            GoldenComparisonOptions::default(),
        )
        .expect("compare golden");

        assert_eq!(
            comparison.status,
            EvalScenarioStatus::Passed,
            "golden mismatches for {}: {:?}",
            scenario_report.scenario_id,
            comparison.mismatches
        );
    }
}
