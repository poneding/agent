use agent_eval::{
    EVAL_RUN_REPORT_SCHEMA_VERSION, EvalArtifactOptions, EvalEvidenceSource, EvalEvidenceSummary,
    EvalReportArtifactError, EvalRunReport, EvalRunSummary, EvalScenarioReport, EvalScenarioStatus,
    write_eval_artifacts,
};
use camino::Utf8PathBuf;

fn scenario(id: &str, status: EvalScenarioStatus) -> EvalScenarioReport {
    EvalScenarioReport {
        scenario_id: id.to_string(),
        status,
        diagnostics: Vec::new(),
        evidence_summary: Some(EvalEvidenceSummary {
            source: EvalEvidenceSource::ReplayTrace,
            event_count: 9,
            checkpoint_count: 1,
            trace_entries: 9,
            audit_records: 9,
            telemetry_records: 0,
            replayable: status == EvalScenarioStatus::Passed,
            diagnostics: 0,
        }),
    }
}

fn sample_report() -> EvalRunReport {
    let scenarios = vec![
        scenario("m04-replay-smoke", EvalScenarioStatus::Passed),
        scenario("missing-fixture", EvalScenarioStatus::Failed),
    ];
    EvalRunReport {
        schema_version: EVAL_RUN_REPORT_SCHEMA_VERSION,
        run_id: "eval-run".to_string(),
        summary: EvalRunSummary::from_reports(&scenarios),
        scenarios,
    }
}

#[test]
fn artifact_writer_creates_json_and_summary_files() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let output_dir =
        Utf8PathBuf::from_path_buf(tempdir.path().join("artifacts")).expect("utf8 path");
    let report = sample_report();

    let artifacts = write_eval_artifacts(
        &report,
        EvalArtifactOptions {
            output_dir: output_dir.clone(),
            write_json: true,
            write_summary: true,
        },
    )
    .expect("write artifacts");

    let json_path = artifacts.json_report.expect("json path");
    let summary_path = artifacts.summary_report.expect("summary path");
    assert_eq!(json_path, output_dir.join("eval-run-report.json"));
    assert_eq!(summary_path, output_dir.join("eval-summary.txt"));

    let parsed: EvalRunReport =
        serde_json::from_str(&std::fs::read_to_string(json_path.as_std_path()).expect("read json"))
            .expect("parse json report");
    assert_eq!(parsed, report);

    let summary = std::fs::read_to_string(summary_path.as_std_path()).expect("read summary");
    assert!(summary.contains("Eval run eval-run"));
    assert!(summary.contains("total=2 passed=1 failed=1 skipped=0 invalid=0 not_run=0"));
    assert!(summary.contains("missing-fixture status=failed"));
}

#[test]
fn artifact_writer_handles_empty_runs() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let report = EvalRunReport {
        schema_version: EVAL_RUN_REPORT_SCHEMA_VERSION,
        run_id: "empty-run".to_string(),
        scenarios: Vec::new(),
        summary: EvalRunSummary::default(),
    };

    let artifacts = write_eval_artifacts(
        &report,
        EvalArtifactOptions {
            output_dir: Utf8PathBuf::from_path_buf(tempdir.path().join("empty"))
                .expect("utf8 path"),
            write_json: true,
            write_summary: true,
        },
    )
    .expect("write empty artifacts");

    let parsed: EvalRunReport = serde_json::from_str(
        &std::fs::read_to_string(artifacts.json_report.expect("json").as_std_path())
            .expect("read json"),
    )
    .expect("parse empty report");
    assert_eq!(parsed.summary.total, 0);
    assert!(parsed.scenarios.is_empty());
}

#[test]
fn artifact_writer_rejects_no_selected_artifacts() {
    let error = write_eval_artifacts(
        &sample_report(),
        EvalArtifactOptions {
            output_dir: Utf8PathBuf::from("target/eval-artifacts"),
            write_json: false,
            write_summary: false,
        },
    )
    .expect_err("no artifacts fails");

    assert_eq!(error, EvalReportArtifactError::NoArtifactsSelected);
}

#[test]
fn artifact_writer_rejects_parent_traversal_output_path() {
    let error = write_eval_artifacts(
        &sample_report(),
        EvalArtifactOptions {
            output_dir: Utf8PathBuf::from("../eval-artifacts"),
            write_json: true,
            write_summary: true,
        },
    )
    .expect_err("unsafe path fails");

    assert!(matches!(
        error,
        EvalReportArtifactError::UnsafeOutputDir { .. }
    ));
}
