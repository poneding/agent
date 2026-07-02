use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_cli::{
    CliCommand, EvalCliCommand, EvalSuiteCliCommand, OutputMode, parse_cli_command,
    parse_cli_invocation,
};
use serde_json::Value;

fn default_suite_path() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/eval/default.suite.json");
    fs::canonicalize(path).expect("canonical default suite")
}

fn default_golden_path() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/eval/goldens/default-m04-replay.golden.json");
    fs::canonicalize(path).expect("canonical default golden")
}

#[test]
fn eval_suite_commands_parse_subcommands() {
    assert_eq!(
        parse_cli_command(["eval", "suite", "list", "--suite", "suite.json"])
            .expect("parse suite list"),
        CliCommand::Eval(EvalCliCommand::Suite(EvalSuiteCliCommand::List {
            suite: PathBuf::from("suite.json")
        }))
    );
    assert_eq!(
        parse_cli_command([
            "eval",
            "suite",
            "update-goldens",
            "--suite",
            "suite.json",
            "--dry-run",
        ])
        .expect("parse update"),
        CliCommand::Eval(EvalCliCommand::Suite(EvalSuiteCliCommand::UpdateGoldens {
            suite: PathBuf::from("suite.json"),
            dry_run: true,
        }))
    );

    let invocation =
        parse_cli_invocation(["--json", "eval", "suite", "run", "--suite", "suite.json"])
            .expect("parse json suite run");
    assert_eq!(invocation.output_mode, OutputMode::Json);
}

#[test]
fn eval_suite_list_text_prints_default_corpus() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["eval", "suite", "list", "--suite"])
        .arg(default_suite_path())
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Eval suite default-regression"));
    assert!(stdout.contains("default-m04-replay"));
    assert!(stdout.contains("default-m04-evidence"));
    assert!(stdout.contains("default-m56-golden-path"));
    assert!(stdout.contains("default-m58-context-quality"));
    assert!(stdout.contains("default-m62-recovery-crash-smoke"));
}

#[test]
fn eval_suite_validate_json_is_parseable() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--json", "eval", "suite", "validate", "--suite"])
        .arg(default_suite_path())
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).expect("json output");
    assert_eq!(value["type"], "eval_suite_validation");
    assert_eq!(value["data"]["valid"], true);
    assert_eq!(value["data"]["suite"]["id"], "default-regression");
    assert_eq!(
        value["data"]["suite"]["scenarios"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
}

#[test]
fn eval_suite_run_text_reports_passed_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["eval", "suite", "run", "--suite"])
        .arg(default_suite_path())
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Eval suite default-regression"));
    assert!(stdout.contains("passed=5"));
    assert!(stdout.contains("default-m04-replay status=passed"));
    assert!(stdout.contains("default-m04-evidence status=passed"));
    assert!(stdout.contains("default-m56-golden-path status=passed"));
    assert!(stdout.contains("default-m58-context-quality status=passed"));
    assert!(stdout.contains("default-m62-recovery-crash-smoke status=passed"));
    assert!(stdout.contains("audit=30"));
}

#[test]
fn eval_suite_update_goldens_dry_run_json_does_not_write() {
    let golden_path = default_golden_path();
    let before = fs::read_to_string(&golden_path).expect("read golden before");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--json", "eval", "suite", "update-goldens", "--suite"])
        .arg(default_suite_path())
        .arg("--dry-run")
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(&golden_path).expect("read golden after"),
        before
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).expect("json output");
    assert_eq!(value["type"], "eval_suite_golden_update");
    assert_eq!(value["data"]["dry_run"], true);
    assert_eq!(value["data"]["report_summary"]["passed"], 5);
    let updated = value["data"]["update"]["updated"].as_u64().unwrap();
    let unchanged = value["data"]["update"]["unchanged"].as_u64().unwrap();
    assert_eq!(updated + unchanged, 5);
}

#[test]
fn eval_suite_invalid_manifest_exits_validation_error() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let suite_path = tempdir.path().join("bad.suite.json");
    fs::write(
        &suite_path,
        r#"{
  "schema_version": 2,
  "id": "bad-suite",
  "name": "Bad suite",
  "default_strictness": "strict",
  "scenarios": []
}
"#,
    )
    .expect("write suite");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["eval", "suite", "validate", "--suite"])
        .arg(&suite_path)
        .output()
        .expect("agent-cli should run");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("eval suite validation failed"));
}
