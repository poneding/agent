use std::fs;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn default_suite_path() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/eval/default.suite.json");
    fs::canonicalize(path).expect("canonical default suite")
}

fn find_m56_scenario(report: &Value) -> &Value {
    find_scenario(report, "default-m56-golden-path")
}

fn find_m58_scenario(report: &Value) -> &Value {
    find_scenario(report, "default-m58-context-quality")
}

fn find_m62_recovery_scenario(report: &Value) -> &Value {
    find_scenario(report, "default-m62-recovery-crash-smoke")
}

fn find_m63_team_scenario(report: &Value) -> &Value {
    find_scenario(report, "default-m63-team-workflow-smoke")
}

fn find_m64_memory_scenario(report: &Value) -> &Value {
    find_scenario(report, "default-m64-memory-smoke")
}

fn find_scenario<'a>(report: &'a Value, scenario_id: &str) -> &'a Value {
    report["scenarios"]
        .as_array()
        .expect("scenarios array")
        .iter()
        .find(|scenario| scenario["scenario_id"] == scenario_id)
        .expect("scenario")
}

#[test]
fn golden_path_e2e_default_suite_json_includes_m56_evidence() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let output_dir = tempdir.path().join("agent-eval");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--json", "eval", "suite", "run", "--suite"])
        .arg(default_suite_path())
        .arg("--out-dir")
        .arg(&output_dir)
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("sk-live-secret"));
    let value: Value = serde_json::from_str(stdout.trim()).expect("json output");
    assert_eq!(value["type"], "eval_suite_run");
    assert_eq!(value["data"]["report"]["summary"]["total"], 7);
    assert_eq!(value["data"]["report"]["summary"]["passed"], 7);

    let scenario = find_m56_scenario(&value["data"]["report"]);
    assert_eq!(scenario["status"], "passed");
    assert_eq!(scenario["diagnostics"].as_array().unwrap().len(), 0);
    assert_eq!(scenario["evidence_summary"]["source"], "local_coding");
    assert_eq!(scenario["evidence_summary"]["audit_records"], 30);
    assert_eq!(scenario["evidence_summary"]["replayable"], true);
    assert_eq!(scenario["evidence_summary"]["diagnostics"], 0);
    let m58_scenario = find_m58_scenario(&value["data"]["report"]);
    assert_eq!(m58_scenario["status"], "passed");
    assert_eq!(
        m58_scenario["evidence_summary"]["source"],
        "context_quality"
    );
    assert_eq!(m58_scenario["evidence_summary"]["replayable"], true);
    let m62_scenario = find_m62_recovery_scenario(&value["data"]["report"]);
    assert_eq!(m62_scenario["status"], "passed");
    assert_eq!(
        m62_scenario["evidence_summary"]["source"],
        "recovery_crash_smoke"
    );
    assert_eq!(m62_scenario["evidence_summary"]["checkpoint_count"], 1);
    assert_eq!(m62_scenario["evidence_summary"]["replayable"], true);
    let m63_scenario = find_m63_team_scenario(&value["data"]["report"]);
    assert_eq!(m63_scenario["status"], "passed");
    assert_eq!(
        m63_scenario["evidence_summary"]["source"],
        "team_workflow_smoke"
    );
    assert_eq!(m63_scenario["evidence_summary"]["event_count"], 4);
    assert_eq!(m63_scenario["evidence_summary"]["audit_records"], 4);
    assert_eq!(m63_scenario["evidence_summary"]["replayable"], true);
    let m64_scenario = find_m64_memory_scenario(&value["data"]["report"]);
    assert_eq!(m64_scenario["status"], "passed");
    assert_eq!(m64_scenario["evidence_summary"]["source"], "memory_smoke");
    assert_eq!(m64_scenario["evidence_summary"]["event_count"], 2);
    assert_eq!(m64_scenario["evidence_summary"]["audit_records"], 3);
    assert_eq!(m64_scenario["evidence_summary"]["replayable"], true);

    let report_path = output_dir.join("eval-run-report.json");
    let artifact: Value =
        serde_json::from_str(&fs::read_to_string(report_path).expect("read eval report"))
            .expect("artifact json");
    let artifact_scenario = find_m56_scenario(&artifact);
    assert_eq!(artifact_scenario["status"], "passed");
    assert_eq!(
        artifact_scenario["evidence_summary"]["source"],
        "local_coding"
    );
    let artifact_recovery = find_m62_recovery_scenario(&artifact);
    assert_eq!(artifact_recovery["status"], "passed");
    let artifact_team = find_m63_team_scenario(&artifact);
    assert_eq!(artifact_team["status"], "passed");
}
