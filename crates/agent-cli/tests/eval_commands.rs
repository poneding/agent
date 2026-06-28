use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use agent_cli::{CliCommand, EvalCliCommand, OutputMode, parse_cli_command, parse_cli_invocation};
use serde_json::{Value, json};

const M04_TRACE_V1: &str = include_str!("../../agent-storage/tests/fixtures/m04_trace_v1.json");

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write file");
}

fn replay_manifest(root: &Path) -> PathBuf {
    write_file(&root.join("traces/m04_trace_v1.json"), M04_TRACE_V1);
    let manifest = json!({
        "schema_version": 1,
        "id": "m04-replay-smoke",
        "name": "M04 replay smoke",
        "kind": "replay_trace",
        "fixtures": {
            "replay_trace": "traces/m04_trace_v1.json"
        },
        "permissions": {},
        "expectations": {
            "expected_status": "passed",
            "required_events": ["final_response_produced", "turn_completed"]
        }
    });
    let path = root.join("m04-replay.eval.json");
    write_file(
        &path,
        &serde_json::to_string_pretty(&manifest).expect("manifest json"),
    );
    path
}

fn local_coding_manifest(root: &Path) -> PathBuf {
    write_file(&root.join("workspace/README.md"), "# fixture\n");
    let manifest = json!({
        "schema_version": 1,
        "id": "local-coding-hook",
        "name": "Local coding hook",
        "kind": "local_coding",
        "fixtures": {
            "workspace": "workspace"
        },
        "permissions": {},
        "expectations": {
            "expected_status": "passed"
        }
    });
    let path = root.join("local-coding.eval.json");
    write_file(
        &path,
        &serde_json::to_string_pretty(&manifest).expect("manifest json"),
    );
    path
}

#[test]
fn eval_commands_parse_subcommands() {
    assert_eq!(
        parse_cli_command(["eval", "list", "--manifest", "scenario.json"]).expect("parse list"),
        CliCommand::Eval(EvalCliCommand::List {
            manifest: PathBuf::from("scenario.json")
        })
    );
    assert_eq!(
        parse_cli_command(["eval", "run", "--manifest", "scenario.json", "--warn"])
            .expect("parse run"),
        CliCommand::Eval(EvalCliCommand::Run {
            manifest: PathBuf::from("scenario.json"),
            strictness: agent_eval::EvalStrictness::Warn,
        })
    );

    let invocation = parse_cli_invocation(["--json", "eval", "validate", "--manifest", "s.json"])
        .expect("parse json validate");
    assert_eq!(invocation.output_mode, OutputMode::Json);
}

#[test]
fn eval_list_cli_prints_scenario() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let manifest = replay_manifest(tempdir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["eval", "list", "--manifest"])
        .arg(&manifest)
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("m04-replay-smoke"));
    assert!(stdout.contains("replay_trace"));
}

#[test]
fn eval_validate_json_is_parseable() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let manifest = replay_manifest(tempdir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--json", "eval", "validate", "--manifest"])
        .arg(&manifest)
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).expect("json output");
    assert_eq!(value["type"], "eval_validation");
    assert_eq!(value["data"]["valid"], true);
    assert_eq!(value["data"]["scenario_id"], "m04-replay-smoke");
}

#[test]
fn eval_run_text_reports_passed_summary() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let manifest = replay_manifest(tempdir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["eval", "run", "--manifest"])
        .arg(&manifest)
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Eval run eval-run"));
    assert!(stdout.contains("passed=1"));
    assert!(stdout.contains("m04-replay-smoke status=passed"));
}

#[test]
fn eval_run_json_reports_summary() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let manifest = replay_manifest(tempdir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--json", "eval", "run", "--manifest"])
        .arg(&manifest)
        .output()
        .expect("agent-cli should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(stdout.trim()).expect("json output");
    assert_eq!(value["type"], "eval_run");
    assert_eq!(value["data"]["report"]["summary"]["passed"], 1);
}

#[test]
fn eval_invalid_manifest_fails_without_secret_value() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let manifest = json!({
        "schema_version": 1,
        "id": "bad-secret",
        "name": "Bad secret",
        "kind": "replay_trace",
        "fixtures": {
            "replay_trace": "missing.json"
        },
        "permissions": {},
        "expectations": {
            "expected_status": "passed"
        },
        "metadata": {
            "api_key": "sk-live-secret-value"
        }
    });
    let path = tempdir.path().join("bad.eval.json");
    write_file(
        &path,
        &serde_json::to_string_pretty(&manifest).expect("manifest json"),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["eval", "validate", "--manifest"])
        .arg(&path)
        .output()
        .expect("agent-cli should run");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("eval validation failed"));
    assert!(!stdout.contains("sk-live-secret-value"));
}

#[test]
fn eval_warn_skipped_exits_success_but_strict_invalid_fails() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let manifest = local_coding_manifest(tempdir.path());

    let warn = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["eval", "run", "--manifest"])
        .arg(&manifest)
        .arg("--warn")
        .output()
        .expect("agent-cli should run");
    assert!(warn.status.success());
    assert!(String::from_utf8_lossy(&warn.stdout).contains("skipped=1"));

    let strict = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["eval", "run", "--manifest"])
        .arg(&manifest)
        .arg("--strict")
        .output()
        .expect("agent-cli should run");
    assert!(!strict.status.success());
    assert_eq!(strict.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&strict.stdout).contains("invalid=1"));
}
