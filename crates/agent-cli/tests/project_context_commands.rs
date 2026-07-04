use std::fs;
use std::process::Command;

use agent_cli::{CliCommand, OutputMode, parse_cli_invocation, render_cli_invocation};
use agent_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

#[test]
fn project_context_commands_parse_text_and_json_forms() {
    let text =
        parse_cli_invocation(["repo", "context", "--workspace", "."]).expect("parse context");
    assert_eq!(text.output_mode, OutputMode::Text);
    assert!(matches!(text.command, CliCommand::RepoContext { .. }));

    let json = parse_cli_invocation(["--json", "repo", "context", "--workspace", "."])
        .expect("parse json context");
    assert_eq!(json.output_mode, OutputMode::Json);
    assert!(matches!(json.command, CliCommand::RepoContext { .. }));
}

#[test]
fn project_context_text_renders_safe_summary() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("src dir");
    fs::write(temp.path().join("src/lib.rs"), "pub fn demo() {}\n").expect("lib");
    fs::write(temp.path().join(".env"), "api_key = \"sk-test-secret\"\n").expect("env");

    let output = render_cli_invocation(
        parse_cli_invocation([
            "repo",
            "context",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
        ])
        .expect("parse context"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Project context: ready"));
    assert!(output.stdout.contains("Workspace: ."));
    assert!(output.stdout.contains("Files: 2"));
    assert!(output.stdout.contains("src/lib.rs"));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn project_context_json_emits_snapshot_and_done() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("src dir");
    fs::write(temp.path().join("src/lib.rs"), "pub fn demo() {}\n").expect("lib");

    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "repo",
            "context",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
        ])
        .expect("parse context"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ProjectContextSnapshot(snapshot))
            if snapshot.workspace_root_display == "."
                && snapshot.file_count == 1
                && snapshot.files.iter().any(|file| file.path == "src/lib.rs")
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count == 0
    ));
}

#[test]
fn project_context_invalid_workspace_exits_two() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "repo",
            "context",
            "--workspace",
            missing.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("agent-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
}
