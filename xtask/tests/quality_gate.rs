use xtask::{
    QualityGateReport, QualityGateStatus, QualityGateStepReport, QualityGateStepStatus,
    render_quality_gate_summary,
};

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
