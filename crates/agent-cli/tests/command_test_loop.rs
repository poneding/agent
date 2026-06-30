use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agent_cli::{CliCommand, OutputMode, parse_cli_invocation, render_cli_invocation};
use agent_ui_bridge::UiJsonFrame;
use tempfile::TempDir;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

fn compile_helper(temp: &TempDir) -> PathBuf {
    let source = temp.path().join("cli_command_helper.rs");
    fs::write(
        &source,
        r#"
use std::env;
use std::process;

fn main() {
    match env::args().nth(1).as_deref() {
        Some("ok") => print!("cli stdout\n"),
        Some("fail") => {
            eprint!("cli stderr\n");
            process::exit(7);
        }
        Some("secret") => print!("api_key sk-test-secret password\n"),
        _ => process::exit(64),
    }
}
"#,
    )
    .expect("write helper source");
    let binary = temp.path().join(format!(
        "cli-command-helper{}",
        std::env::consts::EXE_SUFFIX
    ));
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let status = Command::new(rustc)
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .status()
        .expect("compile helper");
    assert!(status.success(), "helper program should compile");
    binary
}

fn workspace() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir(temp.path().join("subdir")).expect("subdir");
    let helper = compile_helper(&temp);
    (temp, helper)
}

#[test]
fn command_test_loop_parses_text_and_json_forms() {
    let text = parse_cli_invocation(["repo", "command", "--workspace", ".", "--", "cargo", "test"])
        .expect("parse text repo command");
    assert_eq!(text.output_mode, OutputMode::Text);
    assert!(matches!(text.command, CliCommand::RepoCommand { .. }));

    let json = parse_cli_invocation([
        "--json",
        "repo",
        "test",
        "--workspace",
        ".",
        "--timeout-ms",
        "30000",
        "--",
        "cargo",
        "test",
    ])
    .expect("parse json repo test");
    assert_eq!(json.output_mode, OutputMode::Json);
    assert!(matches!(
        json.command,
        CliCommand::RepoCommand {
            classify_as_test: true,
            ..
        }
    ));
}

#[test]
fn command_test_loop_text_command_success() {
    let (temp, helper) = workspace();
    let invocation = parse_cli_invocation([
        "repo",
        "command",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
        "--cwd",
        "subdir",
        "--",
        helper.to_str().expect("utf8 path"),
        "ok",
    ])
    .expect("parse repo command");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Repo command: succeeded"));
    assert!(output.stdout.contains("Classification: none"));
    assert!(output.stdout.contains("Exit: 0"));
    assert!(output.stdout.contains("cli stdout"));
}

#[test]
fn command_test_loop_text_test_failure_classifies_non_zero() {
    let (temp, helper) = workspace();
    let invocation = parse_cli_invocation([
        "repo",
        "test",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
        "--",
        helper.to_str().expect("utf8 path"),
        "fail",
    ])
    .expect("parse repo test");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Repo command: failed"));
    assert!(output.stdout.contains("Classification: test_failure"));
    assert!(output.stdout.contains("Exit: 7"));
    assert!(output.stdout.contains("cli stderr"));
}

#[test]
fn command_test_loop_json_emits_snapshot_events_and_done() {
    let (temp, helper) = workspace();
    let invocation = parse_cli_invocation([
        "--json",
        "repo",
        "test",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
        "--",
        helper.to_str().expect("utf8 path"),
        "fail",
    ])
    .expect("parse repo test");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::CommandTestSnapshot(snapshot))
            if snapshot.status == "failed"
                && snapshot.classification == "test_failure"
                && snapshot.exit_code == Some(7)
                && snapshot.stderr.contains("cli stderr")
    ));
    let event_count = frames
        .iter()
        .filter(|frame| matches!(frame, UiJsonFrame::Event(_)))
        .count();
    assert!(event_count >= 5);
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count: done_count }) if *done_count as usize == event_count
    ));
}

#[test]
fn command_test_loop_invalid_workspace_exits_two() {
    let (temp, helper) = workspace();
    let missing = temp.path().join("missing");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "repo",
            "command",
            "--workspace",
            missing.to_str().expect("utf8 path"),
            "--",
            helper.to_str().expect("utf8 path"),
            "ok",
        ])
        .output()
        .expect("agent-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn command_test_loop_json_redacts_secret_like_output() {
    let (temp, helper) = workspace();
    let invocation = parse_cli_invocation([
        "--json",
        "repo",
        "command",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
        "--",
        helper.to_str().expect("utf8 path"),
        "secret",
    ])
    .expect("parse repo command");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
    assert!(!output.stdout.contains("password"));
    assert!(output.stdout.contains("[redacted]"));
}
