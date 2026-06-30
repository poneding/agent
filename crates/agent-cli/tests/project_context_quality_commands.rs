use std::fs;
use std::process::Command;

use agent_cli::{parse_cli_invocation, render_cli_invocation};
use agent_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("src dir");
    fs::create_dir_all(temp.path().join("docs")).expect("docs dir");
    fs::write(temp.path().join("README.md"), "# Demo\n").expect("readme");
    fs::write(
        temp.path().join("src/command.rs"),
        "pub fn command_palette() {}\n",
    )
    .expect("command");
    fs::write(temp.path().join("src/http.rs"), "pub fn request() {}\n").expect("http");
    fs::write(
        temp.path().join("docs/design.md"),
        format!("# Design\n{}\n", "large design note ".repeat(40)),
    )
    .expect("design");
    fs::write(temp.path().join(".env"), "api_key = \"sk-test-secret\"\n").expect("env");
    temp
}

#[test]
fn project_context_select_without_quality_trace_preserves_default_text() {
    let temp = fixture();
    let output = render_cli_invocation(
        parse_cli_invocation([
            "repo",
            "context",
            "select",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
            "--task",
            "command palette",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Project context selection: ready"));
    assert!(!output.stdout.contains("Context quality: ready"));
}

#[test]
fn project_context_select_quality_trace_text_lists_signals() {
    let temp = fixture();
    let output = render_cli_invocation(
        parse_cli_invocation([
            "repo",
            "context",
            "select",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
            "--task",
            "command sk-test-secret palette",
            "--quality-trace",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Project context selection: ready"));
    assert!(output.stdout.contains("Context quality: ready"));
    assert!(output.stdout.contains("Retrieval kind: lexical"));
    assert!(output.stdout.contains("Budget pressure:"));
    assert!(output.stdout.contains("Signals:"));
    assert!(output.stdout.contains("src/command.rs"));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
}

#[test]
fn project_context_select_quality_trace_json_emits_snapshot_and_done() {
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
            "command sk-test-secret palette",
            "--quality-trace",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ProjectContextRetrievalSnapshot(_))
    ));
    assert!(matches!(
        frames.get(1),
        Some(UiJsonFrame::ProjectContextQualityTraceSnapshot(snapshot))
            if snapshot.retrieval_kind == "lexical"
                && snapshot.task == "command [redacted] palette"
                && snapshot.selected_count > 0
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count == 0
    ));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn project_context_select_quality_trace_json_covers_pinned_hybrid() {
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
            "command palette",
            "--semantic",
            "fixture",
            "--required-pin",
            "README.md",
            "--pin-reason",
            "entry api_key",
            "--context-budget",
            "400",
            "--reserved-response-tokens",
            "50",
            "--quality-trace",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ProjectContextPinnedRetrievalSnapshot(snapshot))
            if snapshot.retrieval_kind == "hybrid"
    ));
    assert!(matches!(
        frames.get(1),
        Some(UiJsonFrame::ProjectContextQualityTraceSnapshot(snapshot))
            if snapshot.retrieval_kind == "pinned_hybrid"
                && snapshot.pinned_included_count == 1
                && snapshot.signals.iter().any(|signal| signal.kind == "semantic_completed")
    ));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn project_context_select_quality_trace_invalid_workspace_exits_two() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args([
            "repo",
            "context",
            "select",
            "--workspace",
            missing.to_str().expect("utf8 path"),
            "--task",
            "command palette",
            "--quality-trace",
        ])
        .output()
        .expect("agent-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
}
