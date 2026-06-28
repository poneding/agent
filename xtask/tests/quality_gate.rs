use xtask::{
    CargoCommand, QualityGateError, QualityGateOptions, QualityGateReport, QualityGateStatus,
    QualityGateStepReport, QualityGateStepStatus, render_quality_gate_summary,
    run_quality_gate_commands,
};

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
    let json_path = output_dir.join("quality-gate-report.json");
    let summary_path = output_dir.join("quality-gate-summary.txt");
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
    assert!(output_dir.join("quality-gate-report.json").exists());
    assert!(output_dir.join("quality-gate-summary.txt").exists());
}
