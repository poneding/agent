use xtask::{
    CargoCommand, QUALITY_GATE_ARTIFACT_DIR, QUALITY_GATE_ARTIFACT_FILES, QUALITY_GATE_REPORT_FILE,
    QUALITY_GATE_SUMMARY_FILE, QualityGateError, QualityGateOptions, QualityGateReport,
    QualityGateStatus, QualityGateStepReport, QualityGateStepStatus, render_quality_gate_summary,
    run_quality_gate_commands, write_quality_gate_artifacts,
};

const README: &str = include_str!("../../README.md");
const CI_WORKFLOW: &str = include_str!("../../.github/workflows/ci.yml");

fn temp_output_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agent-xtask-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    if path.exists() {
        std::fs::remove_dir_all(&path).expect("remove stale tempdir");
    }
    path
}

fn step(
    name: &str,
    status: QualityGateStepStatus,
    exit_code: Option<i32>,
) -> QualityGateStepReport {
    QualityGateStepReport::new(
        name,
        vec!["cargo".to_string(), "test".to_string()],
        status,
        exit_code,
        42,
        None,
    )
}

#[test]
fn quality_gate_artifact_contract_exposes_stable_paths() {
    assert_eq!(
        QualityGateOptions::default().output_dir,
        std::path::PathBuf::from(QUALITY_GATE_ARTIFACT_DIR)
    );
    assert_eq!(QUALITY_GATE_ARTIFACT_DIR, "target/agent-ci");
    assert_eq!(QUALITY_GATE_REPORT_FILE, "quality-gate-report.json");
    assert_eq!(QUALITY_GATE_SUMMARY_FILE, "quality-gate-summary.txt");
    assert_eq!(
        QUALITY_GATE_ARTIFACT_FILES,
        [QUALITY_GATE_REPORT_FILE, QUALITY_GATE_SUMMARY_FILE]
    );
}

#[test]
fn quality_gate_artifact_paths_are_documented_for_ci_upload() {
    assert!(README.contains(QUALITY_GATE_ARTIFACT_DIR));
    assert!(README.contains(QUALITY_GATE_REPORT_FILE));
    assert!(README.contains(QUALITY_GATE_SUMMARY_FILE));
    assert!(CI_WORKFLOW.contains(&format!("path: {QUALITY_GATE_ARTIFACT_DIR}")));
}

#[test]
fn quality_gate_report_serializes_stable_shape() {
    let report = QualityGateReport::from_steps(
        "ci-local",
        vec![step("test", QualityGateStepStatus::Passed, Some(0))],
    );

    let value = serde_json::to_value(&report).expect("report json");

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["run_id"], "ci-local");
    assert_eq!(value["status"], "passed");
    assert_eq!(value["steps"][0]["name"], "test");
    assert_eq!(value["steps"][0]["command"][0], "cargo");
    assert_eq!(value["steps"][0]["status"], "passed");
    assert_eq!(value["steps"][0]["exit_code"], 0);
    assert_eq!(value["steps"][0]["duration_ms"], 42);
}

#[test]
fn quality_gate_report_status_fails_when_any_step_fails() {
    let report = QualityGateReport::from_steps(
        "ci-local",
        vec![
            step("fmt", QualityGateStepStatus::Passed, Some(0)),
            step("clippy", QualityGateStepStatus::Failed, Some(101)),
            step("test", QualityGateStepStatus::Skipped, None),
        ],
    );

    assert_eq!(report.status, QualityGateStatus::Failed);
}

#[test]
fn quality_gate_summary_renders_counts_and_steps() {
    let report = QualityGateReport::from_steps(
        "ci-local",
        vec![
            step("fmt", QualityGateStepStatus::Passed, Some(0)),
            step("clippy", QualityGateStepStatus::Failed, Some(101)),
            step("test", QualityGateStepStatus::Skipped, None),
        ],
    );

    let summary = render_quality_gate_summary(&report);

    assert!(summary.contains("Quality gate ci-local status=failed"));
    assert!(summary.contains("total=3 passed=1 failed=1 skipped=1"));
    assert!(summary.contains("fmt status=passed exit=0 duration_ms=42"));
    assert!(summary.contains("clippy status=failed exit=101 duration_ms=42"));
    assert!(summary.contains("test status=skipped exit=- duration_ms=42"));
}

#[test]
fn quality_gate_messages_are_redacted_and_bounded() {
    let report = QualityGateReport::from_steps(
        "ci-local",
        vec![QualityGateStepReport::new(
            "eval",
            vec!["cargo".to_string(), "xtask".to_string(), "eval".to_string()],
            QualityGateStepStatus::Failed,
            Some(1),
            7,
            Some(format!("api_key sk-live-secret-value {}", "x".repeat(400))),
        )],
    );

    let serialized = serde_json::to_string(&report).expect("report json");
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-live-secret-value"));
    assert!(serialized.len() < 600);
}

#[test]
fn quality_gate_runner_writes_json_and_summary_artifacts() {
    let output_dir = temp_output_dir("quality-pass");

    let report = run_quality_gate_commands(
        QualityGateOptions {
            run_id: "fixture-ci".to_string(),
            output_dir: output_dir.clone(),
        },
        vec![CargoCommand {
            program: "rustc".to_string(),
            args: vec!["--version".to_string()],
        }],
    )
    .expect("quality gate should pass");

    assert_eq!(report.status, QualityGateStatus::Passed);
    let json_path = output_dir.join(QUALITY_GATE_REPORT_FILE);
    let summary_path = output_dir.join(QUALITY_GATE_SUMMARY_FILE);
    assert!(json_path.exists());
    assert!(summary_path.exists());
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(json_path).expect("read json"))
            .expect("json report");
    assert_eq!(value["run_id"], "fixture-ci");
    assert!(
        std::fs::read_to_string(summary_path)
            .expect("read summary")
            .contains("Quality gate fixture-ci status=passed")
    );
}

#[test]
fn quality_gate_runner_stops_after_failure_and_records_skipped_steps() {
    let output_dir = temp_output_dir("quality-fail");

    let error = run_quality_gate_commands(
        QualityGateOptions {
            run_id: "fixture-ci".to_string(),
            output_dir: output_dir.clone(),
        },
        vec![
            CargoCommand {
                program: "rustc".to_string(),
                args: vec!["--definitely-not-a-real-rustc-flag".to_string()],
            },
            CargoCommand {
                program: "rustc".to_string(),
                args: vec!["--version".to_string()],
            },
        ],
    )
    .expect_err("quality gate should fail");

    let QualityGateError::CommandFailed { report, code } = error else {
        panic!("expected command failure");
    };
    assert_ne!(code, 0);
    assert_eq!(report.status, QualityGateStatus::Failed);
    assert_eq!(report.steps[0].status, QualityGateStepStatus::Failed);
    assert_eq!(report.steps[1].status, QualityGateStepStatus::Skipped);
    assert!(output_dir.join(QUALITY_GATE_REPORT_FILE).exists());
    assert!(output_dir.join(QUALITY_GATE_SUMMARY_FILE).exists());
}

#[test]
fn quality_gate_runner_records_spawn_failure_partial_report() {
    let output_dir = temp_output_dir("quality-spawn-fail");

    let error = run_quality_gate_commands(
        QualityGateOptions {
            run_id: "fixture-ci".to_string(),
            output_dir: output_dir.clone(),
        },
        vec![
            CargoCommand {
                program: format!("agent-missing-command-{}", std::process::id()),
                args: Vec::new(),
            },
            CargoCommand {
                program: "rustc".to_string(),
                args: vec!["--version".to_string()],
            },
        ],
    )
    .expect_err("quality gate should fail before spawning missing command");

    let QualityGateError::CommandSpawn { report, .. } = error else {
        panic!("expected command spawn failure");
    };
    assert_eq!(report.status, QualityGateStatus::Failed);
    assert_eq!(report.steps[0].status, QualityGateStepStatus::Failed);
    assert_eq!(report.steps[1].status, QualityGateStepStatus::Skipped);

    let value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(output_dir.join(QUALITY_GATE_REPORT_FILE)).expect("read json"),
    )
    .expect("json report");
    assert_eq!(value["status"], "failed");
    assert_eq!(value["steps"][1]["status"], "skipped");
    assert!(
        std::fs::read_to_string(output_dir.join(QUALITY_GATE_SUMMARY_FILE))
            .expect("read summary")
            .contains("status=failed")
    );
}

#[test]
fn quality_gate_artifact_writer_overwrites_stale_files() {
    let output_dir = temp_output_dir("quality-overwrite");
    std::fs::create_dir_all(&output_dir).expect("create output dir");
    std::fs::write(output_dir.join(QUALITY_GATE_REPORT_FILE), "stale json")
        .expect("write stale json");
    std::fs::write(output_dir.join(QUALITY_GATE_SUMMARY_FILE), "stale summary")
        .expect("write stale summary");
    let report = QualityGateReport::from_steps(
        "fixture-ci",
        vec![step("fmt", QualityGateStepStatus::Passed, Some(0))],
    );

    write_quality_gate_artifacts(&report, &output_dir).expect("write artifacts");

    let json = std::fs::read_to_string(output_dir.join(QUALITY_GATE_REPORT_FILE))
        .expect("read json artifact");
    let summary = std::fs::read_to_string(output_dir.join(QUALITY_GATE_SUMMARY_FILE))
        .expect("read summary artifact");
    assert!(!json.contains("stale json"));
    assert!(!summary.contains("stale summary"));
    assert!(json.ends_with('\n'));
    assert!(summary.contains("Quality gate fixture-ci status=passed"));
}
