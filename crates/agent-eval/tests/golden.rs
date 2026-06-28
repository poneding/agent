use agent_eval::{
    EvalDiagnostic, EvalDiagnosticSeverity, EvalEvidenceSource, EvalEvidenceSummary,
    EvalScenarioReport, EvalScenarioStatus, GoldenComparisonError, GoldenComparisonOptions,
    compare_report_to_golden_file, compare_report_to_golden_json,
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
