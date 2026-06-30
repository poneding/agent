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
fn edit_diff_flow_parses_text_and_json_forms() {
    let text = parse_cli_invocation([
        "repo",
        "edit",
        "--workspace",
        ".",
        "--path",
        "README.md",
        "--replacement-file",
        "after.txt",
        "--preview",
    ])
    .expect("parse text repo edit");
    assert_eq!(text.output_mode, OutputMode::Text);
    assert!(matches!(text.command, CliCommand::RepoEdit { .. }));

    let json = parse_cli_invocation([
        "--json",
        "repo",
        "edit",
        "--workspace",
        ".",
        "--path",
        "README.md",
        "--replacement-file",
        "after.txt",
        "--apply",
    ])
    .expect("parse json repo edit");
    assert_eq!(json.output_mode, OutputMode::Json);
    assert!(matches!(json.command, CliCommand::RepoEdit { .. }));
}

#[test]
fn edit_diff_flow_text_preview_does_not_mutate_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("README.md"), "old text\n").expect("write readme");
    let replacement = temp.path().join("after.txt");
    fs::write(&replacement, "new text\n").expect("write replacement");

    let invocation = parse_cli_invocation([
        "repo",
        "edit",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
        "--path",
        "README.md",
        "--replacement-file",
        replacement.to_str().expect("utf8 path"),
        "--preview",
    ])
    .expect("parse repo edit");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Repo edit: preview"));
    assert!(output.stdout.contains("Path: README.md"));
    assert!(output.stdout.contains("Diff: preview"));
    assert!(output.stdout.contains("+1 -1"));
    assert_eq!(
        fs::read_to_string(temp.path().join("README.md")).expect("read file"),
        "old text\n"
    );
}

#[test]
fn edit_diff_flow_text_apply_mutates_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("README.md"), "old text\n").expect("write readme");
    let expected = temp.path().join("before.txt");
    let replacement = temp.path().join("after.txt");
    fs::write(&expected, "old text\n").expect("write expected");
    fs::write(&replacement, "new text\n").expect("write replacement");

    let invocation = parse_cli_invocation([
        "repo",
        "edit",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
        "--path",
        "README.md",
        "--expected-file",
        expected.to_str().expect("utf8 path"),
        "--replacement-file",
        replacement.to_str().expect("utf8 path"),
        "--apply",
    ])
    .expect("parse repo edit");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Repo edit: applied"));
    assert!(output.stdout.contains("Changed files: README.md"));
    assert_eq!(
        fs::read_to_string(temp.path().join("README.md")).expect("read file"),
        "new text\n"
    );
}

#[test]
fn edit_diff_flow_json_emits_snapshot_events_and_done() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("README.md"), "old text\n").expect("write readme");
    let replacement = temp.path().join("after.txt");
    fs::write(&replacement, "new text\n").expect("write replacement");

    let invocation = parse_cli_invocation([
        "--json",
        "repo",
        "edit",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
        "--path",
        "README.md",
        "--replacement-file",
        replacement.to_str().expect("utf8 path"),
        "--preview",
    ])
    .expect("parse repo edit");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::EditDiffSnapshot(snapshot))
            if !snapshot.applied
                && snapshot.path == "README.md"
                && snapshot.diff_state == "preview"
                && snapshot.added_line_count == 1
                && snapshot.removed_line_count == 1
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
fn edit_diff_flow_invalid_workspace_exits_two() {
    let temp = tempfile::tempdir().expect("tempdir");
    let replacement = temp.path().join("after.txt");
    fs::write(&replacement, "new text\n").expect("write replacement");
    let missing = temp.path().join("missing");

    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "repo",
            "edit",
            "--workspace",
            missing.to_str().expect("utf8 path"),
            "--path",
            "README.md",
            "--replacement-file",
            replacement.to_str().expect("utf8 path"),
            "--preview",
        ])
        .output()
        .expect("agent-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn edit_diff_flow_json_redacts_secret_like_diff() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("README.md"), "old text\n").expect("write readme");
    let replacement = temp.path().join("after.txt");
    fs::write(&replacement, "api_key = \"sk-test-secret\"\n").expect("write replacement");

    let invocation = parse_cli_invocation([
        "--json",
        "repo",
        "edit",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
        "--path",
        "README.md",
        "--replacement-file",
        replacement.to_str().expect("utf8 path"),
        "--preview",
    ])
    .expect("parse repo edit");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
    assert!(!output.stdout.contains("secret"));
    assert!(output.stdout.contains("[redacted]"));
}
