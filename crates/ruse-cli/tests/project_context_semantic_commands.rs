use ruse_ui_bridge::UiProjectContextDetail;
use std::fs;
use std::process::Command;

use ruse_cli::{CliCommand, OutputMode, parse_cli_invocation, render_cli_invocation};
use ruse_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("src dir");
    fs::write(
        temp.path().join("src/command.rs"),
        "pub fn command_palette() {}\n",
    )
    .expect("command");
    fs::write(temp.path().join("src/http.rs"), "pub fn request() {}\n").expect("http");
    fs::write(temp.path().join(".env"), "api_key = \"sk-test-secret\"\n").expect("env");
    temp
}

#[test]
fn project_context_select_parses_semantic_flags() {
    let invocation = parse_cli_invocation([
        "--json",
        "repo",
        "context",
        "select",
        "--workspace",
        ".",
        "--task",
        "update CLI context",
        "--semantic",
        "fixture",
        "--semantic-dimensions",
        "4",
        "--lexical-weight",
        "0.25",
        "--semantic-weight",
        "0.75",
        "--minimum-score",
        "0.1",
        "--semantic-fallback",
        "fail-closed",
    ])
    .expect("parse semantic select");

    assert_eq!(invocation.output_mode, OutputMode::Json);
    assert!(matches!(
        invocation.command,
        CliCommand::RepoContextSelect { .. }
    ));
}

#[test]
fn project_context_select_semantic_off_preserves_lexical_text_output() {
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
            "--semantic",
            "off",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Project context selection: ready"));
    assert!(!output.stdout.contains("Semantic:"));
    assert!(output.stdout.contains("src/command.rs"));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn project_context_select_semantic_fixture_lists_hybrid_scores() {
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
            "--semantic",
            "fixture",
            "--semantic-dimensions",
            "4",
            "--lexical-weight",
            "0.2",
            "--semantic-weight",
            "0.8",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Project context selection: ready"));
    assert!(output.stdout.contains("Semantic: completed"));
    assert!(output.stdout.contains("provider=fixture"));
    assert!(output.stdout.contains("lexical="));
    assert!(output.stdout.contains("semantic="));
    assert!(output.stdout.contains("combined="));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
    assert!(!output.stdout.contains("sk-test-secret"));
}

#[test]
fn project_context_select_semantic_fixture_json_emits_hybrid_snapshot() {
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
            "--semantic",
            "fixture",
            "--semantic-dimensions",
            "4",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ProjectContextDetail(ruse_ui_bridge::UiProjectContextDetail::Hybrid(snapshot)))
            if snapshot.workspace_root_display == "."
                && snapshot.task == "command [redacted] palette"
                && snapshot.semantic.status == "completed"
                && snapshot.semantic.provider_id.as_deref() == Some("fixture")
                && snapshot.selected.iter().any(|candidate| candidate.path == "src/command.rs")
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count == 0
    ));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
}

#[test]
fn project_context_select_invalid_semantic_options_exit_two() {
    for args in [
        vec![
            "repo",
            "context",
            "select",
            "--task",
            "update CLI context",
            "--semantic",
            "fixture",
            "--semantic-dimensions",
            "0",
        ],
        vec![
            "repo",
            "context",
            "select",
            "--task",
            "update CLI context",
            "--semantic",
            "fixture",
            "--lexical-weight",
            "-1",
        ],
        vec![
            "repo",
            "context",
            "select",
            "--task",
            " ",
            "--semantic",
            "fixture",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
            .args(args)
            .output()
            .expect("ruse-cli binary should run");
        assert_eq!(output.status.code(), Some(2));
    }
}
