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

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("crates/agent-cli/src")).expect("cli src");
    fs::create_dir_all(temp.path().join("crates/agent-cli/tests")).expect("cli tests");
    fs::write(
        temp.path().join("crates/agent-cli/src/command.rs"),
        "pub fn context_command() {}\n",
    )
    .expect("command");
    fs::write(
        temp.path()
            .join("crates/agent-cli/tests/project_context_commands.rs"),
        "#[test]\nfn context_command() {}\n",
    )
    .expect("command test");
    fs::write(temp.path().join("README.md"), "# Demo\n").expect("readme");
    fs::write(temp.path().join(".env"), "api_key = \"sk-test-secret\"\n").expect("env");
    temp
}

#[test]
fn project_context_select_parses_text_and_json_forms() {
    let text = parse_cli_invocation([
        "repo",
        "context",
        "select",
        "--workspace",
        ".",
        "--task",
        "update CLI context",
    ])
    .expect("parse select");
    assert_eq!(text.output_mode, OutputMode::Text);
    assert!(matches!(text.command, CliCommand::RepoContextSelect { .. }));

    let json = parse_cli_invocation([
        "--json",
        "repo",
        "context",
        "select",
        "--workspace",
        ".",
        "--task",
        "update CLI context",
    ])
    .expect("parse json select");
    assert_eq!(json.output_mode, OutputMode::Json);
    assert!(matches!(json.command, CliCommand::RepoContextSelect { .. }));
}

#[test]
fn project_context_select_text_lists_selected_files_and_reasons() {
    let temp = fixture();
    let output = render_cli_invocation(
        parse_cli_invocation([
            "repo",
            "context",
            "select",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
            "--task",
            "update rust CLI context command",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Project context selection: ready"));
    assert!(output.stdout.contains("Selected:"));
    assert!(output.stdout.contains("crates/agent-cli/src/command.rs"));
    assert!(output.stdout.contains("path_token_match"));
    assert!(output.stdout.contains("file_name_match"));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn project_context_select_json_emits_retrieval_snapshot_and_done() {
    let temp = fixture();
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "repo",
            "context",
            "select",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
            "--task",
            "update sk-test-secret CLI context command",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ProjectContextRetrievalSnapshot(snapshot))
            if snapshot.workspace_root_display == "."
                && snapshot.task == "update [redacted] CLI context command"
                && snapshot.selected.iter().any(|file| file.path == "crates/agent-cli/src/command.rs")
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count == 0
    ));
    assert!(!output.stdout.contains("sk-test-secret"));
}

#[test]
fn project_context_select_pin_boosts_matching_path() {
    let temp = fixture();
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "repo",
            "context",
            "select",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
            "--task",
            "rust source command",
            "--pin",
            "README.md",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ProjectContextRetrievalSnapshot(snapshot))
            if snapshot.selected.first().map(|file| file.path.as_str()) == Some("README.md")
                && snapshot.selected.first().expect("selected").reasons.contains(&"pinned_path".to_string())
    ));
}

#[test]
fn project_context_select_invalid_workspace_and_empty_task_exit_two() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");

    let invalid_workspace = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "repo",
            "context",
            "select",
            "--workspace",
            missing.to_str().expect("utf8 path"),
            "--task",
            "update CLI context",
        ])
        .output()
        .expect("agent-cli binary should run");
    assert_eq!(invalid_workspace.status.code(), Some(2));

    let empty_task = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["repo", "context", "select", "--task", " "])
        .output()
        .expect("agent-cli binary should run");
    assert_eq!(empty_task.status.code(), Some(2));
}
