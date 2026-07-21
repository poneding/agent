use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use ruse_eval::{
    EVAL_RUN_REPORT_SCHEMA_VERSION, EvalDiagnostic, EvalDiagnosticSeverity, EvalEvidenceSource,
    EvalEvidenceSummary, EvalRunReport, EvalRunSummary, EvalScenarioReport, EvalScenarioStatus,
    GoldenComparisonError, GoldenComparisonOptions, GoldenUpdateError, GoldenUpdateOptions,
    compare_report_to_golden_file, compare_report_to_golden_json, parse_eval_suite_json,
    update_suite_goldens,
};
use serde_json::json;

fn report() -> EvalScenarioReport {
    EvalScenarioReport {
        scenario_id: "scenario-replay".to_string(),
        status: EvalScenarioStatus::Passed,
        diagnostics: vec![EvalDiagnostic {
            severity: EvalDiagnosticSeverity::Warning,
            code: "note".to_string(),
            message: "bounded note".to_string(),
            field: Some("diagnostics".to_string()),
        }],
        evidence_summary: Some(EvalEvidenceSummary {
            source: EvalEvidenceSource::ReplayTrace,
            event_count: 9,
            checkpoint_count: 1,
            trace_entries: 9,
            audit_records: 9,
            telemetry_records: 0,
            replayable: true,
            diagnostics: 0,
        }),
    }
}

fn write_file(root: &Utf8Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent").as_std_path()).expect("create parent");
    fs::write(path.as_std_path(), content).expect("write file");
}

fn update_suite(root: &Utf8Path, golden_path: &str) -> ruse_eval::ValidatedEvalSuite {
    write_file(
        root,
        "scenarios/update.eval.json",
        &json!({
            "schema_version": 1,
            "id": "scenario-replay",
            "name": "Scenario replay",
            "kind": "evidence_only",
            "fixtures": {
                "evidence_bundle": "fixtures/evidence.json"
            },
            "permissions": {},
            "expectations": {
                "expected_status": "passed",
                "golden": golden_path
            }
        })
        .to_string(),
    );

    let suite = json!({
        "schema_version": 1,
        "id": "golden-update",
        "name": "Golden update",
        "default_strictness": "strict",
        "scenarios": [
            {
                "path": "scenarios/update.eval.json",
                "alias": "scenario-replay",
                "labels": ["golden"]
            }
        ]
    })
    .to_string();
    parse_eval_suite_json(&suite, root).expect("valid update suite")
}

fn run_report(report: EvalScenarioReport) -> EvalRunReport {
    let scenarios = vec![report];
    EvalRunReport {
        schema_version: EVAL_RUN_REPORT_SCHEMA_VERSION,
        run_id: "golden-update-test".to_string(),
        summary: EvalRunSummary::from_reports(&scenarios),
        scenarios,
    }
}

fn temp_root(tempdir: &tempfile::TempDir) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf()).expect("utf8 tempdir")
}

#[test]
fn exact_golden_match_passes() {
    let report = report();
    let golden = serde_json::to_string_pretty(&report).expect("serialize golden");

    let result =
        compare_report_to_golden_json(&report, &golden, GoldenComparisonOptions::default())
            .expect("compare");

    assert_eq!(result.status, EvalScenarioStatus::Passed);
    assert!(result.mismatches.is_empty());
    assert!(result.normalized_expected.is_none());
}

#[test]
fn scalar_mismatch_reports_stable_json_pointer_path() {
    let report = report();
    let golden = json!({
        "status": "failed"
    })
    .to_string();

    let result =
        compare_report_to_golden_json(&report, &golden, GoldenComparisonOptions::default())
            .expect("compare");

    assert_eq!(result.status, EvalScenarioStatus::Failed);
    assert_eq!(result.mismatches.len(), 1);
    assert_eq!(result.mismatches[0].path, "/status");
    assert_eq!(result.mismatches[0].expected, "\"failed\"");
    assert_eq!(result.mismatches[0].actual, "\"passed\"");
}

#[test]
fn golden_selects_fields_and_ignores_actual_extras() {
    let report = report();
    let golden = json!({
        "scenario_id": "scenario-replay"
    })
    .to_string();

    let result =
        compare_report_to_golden_json(&report, &golden, GoldenComparisonOptions::default())
            .expect("compare");

    assert_eq!(result.status, EvalScenarioStatus::Passed);
    assert!(result.mismatches.is_empty());
}

#[test]
fn selected_missing_field_is_a_deterministic_mismatch() {
    let report = report();
    let golden = json!({
        "scenario_id": "scenario-replay",
        "evidence_summary": {
            "unknown_count": 10
        }
    })
    .to_string();

    let result =
        compare_report_to_golden_json(&report, &golden, GoldenComparisonOptions::default())
            .expect("compare");

    assert_eq!(result.status, EvalScenarioStatus::Failed);
    assert_eq!(result.mismatches[0].path, "/evidence_summary/unknown_count");
    assert_eq!(result.mismatches[0].actual, "<missing>");
}

#[test]
fn missing_golden_file_fails_without_update_mode() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let path = tempdir.path().join("missing.golden.json");

    let result =
        compare_report_to_golden_file(&report(), &path, GoldenComparisonOptions::default())
            .expect("compare");

    assert_eq!(result.status, EvalScenarioStatus::Failed);
    assert_eq!(result.mismatches[0].path, "");
    assert_eq!(result.mismatches[0].actual, "<missing>");
}

#[test]
fn update_mode_returns_normalized_expected_for_missing_file() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let path = tempdir.path().join("missing.golden.json");
    let report = report();

    let result = compare_report_to_golden_file(
        &report,
        &path,
        GoldenComparisonOptions {
            update_mode: true,
            ..GoldenComparisonOptions::default()
        },
    )
    .expect("compare");

    assert_eq!(result.status, EvalScenarioStatus::Passed);
    assert!(result.mismatches.is_empty());
    assert_eq!(
        result.normalized_expected,
        Some(serde_json::to_value(report).expect("report json"))
    );
}

#[test]
fn malformed_golden_json_returns_error() {
    let error =
        compare_report_to_golden_json(&report(), "{not-json", GoldenComparisonOptions::default())
            .expect_err("bad json fails");

    assert!(matches!(
        error,
        GoldenComparisonError::MalformedGolden { .. }
    ));
}

#[test]
fn mismatch_values_are_redacted_and_bounded() {
    let mut report = report();
    report.diagnostics[0].message = "api_key sk-live-secret-value".to_string();
    let golden = json!({
        "diagnostics": [
            { "message": "plain" }
        ]
    })
    .to_string();

    let result = compare_report_to_golden_json(
        &report,
        &golden,
        GoldenComparisonOptions {
            max_value_chars: 32,
            ..GoldenComparisonOptions::default()
        },
    )
    .expect("compare");

    let mismatch = &result.mismatches[0];
    assert_eq!(mismatch.path, "/diagnostics/0/message");
    assert!(mismatch.actual.contains("[redacted]"));
    assert!(!mismatch.actual.contains("sk-live-secret-value"));
}

#[test]
fn golden_update_dry_run_reports_without_writing() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = temp_root(&tempdir);
    write_file(
        &root,
        "goldens/report.golden.json",
        "{\"status\":\"failed\"}\n",
    );
    let suite = update_suite(&root, "goldens/report.golden.json");

    let update = update_suite_goldens(
        &suite,
        &run_report(report()),
        GoldenUpdateOptions { dry_run: true },
    )
    .expect("dry-run update");

    assert_eq!(update.updated, 1);
    assert_eq!(update.unchanged, 0);
    assert_eq!(update.diagnostics[0].code, "golden_update_dry_run");
    assert_eq!(
        fs::read_to_string(root.join("goldens/report.golden.json").as_std_path())
            .expect("read golden"),
        "{\"status\":\"failed\"}\n"
    );
}

#[test]
fn golden_update_write_creates_newline_terminated_golden() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = temp_root(&tempdir);
    let suite = update_suite(&root, "goldens/report.golden.json");

    let update = update_suite_goldens(
        &suite,
        &run_report(report()),
        GoldenUpdateOptions { dry_run: false },
    )
    .expect("write update");

    assert_eq!(update.updated, 1);
    assert_eq!(update.unchanged, 0);

    let content = fs::read_to_string(root.join("goldens/report.golden.json").as_std_path())
        .expect("read written golden");
    assert!(content.ends_with('\n'));
    let value: serde_json::Value = serde_json::from_str(&content).expect("golden json");
    assert_eq!(value["scenario_id"], "scenario-replay");

    let unchanged = update_suite_goldens(
        &suite,
        &run_report(report()),
        GoldenUpdateOptions { dry_run: false },
    )
    .expect("unchanged update");
    assert_eq!(unchanged.updated, 0);
    assert_eq!(unchanged.unchanged, 1);
}

#[test]
fn golden_update_rejects_malformed_existing_golden_before_overwrite() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = temp_root(&tempdir);
    write_file(&root, "goldens/report.golden.json", "{not-json\n");
    let suite = update_suite(&root, "goldens/report.golden.json");

    let error = update_suite_goldens(
        &suite,
        &run_report(report()),
        GoldenUpdateOptions { dry_run: false },
    )
    .expect_err("malformed golden should fail");

    assert!(matches!(
        error,
        GoldenUpdateError::MalformedExistingGolden { .. }
    ));
    assert_eq!(
        fs::read_to_string(root.join("goldens/report.golden.json").as_std_path())
            .expect("read golden"),
        "{not-json\n"
    );
}

#[test]
fn golden_update_redacts_secret_like_values() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = temp_root(&tempdir);
    let suite = update_suite(&root, "goldens/report.golden.json");
    let mut report = report();
    report.diagnostics[0].message = "api_key sk-live-secret-value".to_string();

    update_suite_goldens(
        &suite,
        &run_report(report),
        GoldenUpdateOptions { dry_run: false },
    )
    .expect("write update");

    let content = fs::read_to_string(root.join("goldens/report.golden.json").as_std_path())
        .expect("read golden");
    assert!(content.contains("[redacted]"));
    assert!(!content.contains("sk-live-secret-value"));
}
