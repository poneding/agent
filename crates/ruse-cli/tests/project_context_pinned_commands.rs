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
    fs::create_dir_all(temp.path().join("docs")).expect("docs dir");
    fs::write(temp.path().join("README.md"), "# Demo\n").expect("readme");
    fs::write(
        temp.path().join("src/command.rs"),
        "pub fn command_palette() {}\n",
    )
    .expect("command");
    fs::write(
        temp.path().join("docs/design.md"),
        format!("# Design\n{}\n", "large design note ".repeat(40)),
    )
    .expect("design");
    fs::write(temp.path().join(".env"), "api_key = \"sk-test-secret\"\n").expect("env");
    temp
}

#[test]
fn project_context_select_parses_pinned_budget_flags() {
    let invocation = parse_cli_invocation([
        "--json",
        "repo",
        "context",
        "select",
        "--workspace",
        ".",
        "--task",
        "update CLI context",
        "--required-pin",
        "README.md",
        "--optional-pin",
        "src/command.rs",
        "--pin-reason",
        "entry point",
        "--context-budget",
        "400",
        "--reserved-response-tokens",
        "50",
    ])
    .expect("parse pinned select");

    assert_eq!(invocation.output_mode, OutputMode::Json);
    assert!(matches!(
        invocation.command,
        CliCommand::RepoContextSelect { .. }
    ));
}

#[test]
fn project_context_select_required_pin_text_includes_budget_evidence() {
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
            "--required-pin",
            "README.md",
            "--pin-reason",
            "entry point",
            "--context-budget",
            "400",
            "--reserved-response-tokens",
            "50",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    assert!(
        output
            .stdout
            .contains("Project context pinned selection: ready")
    );
    assert!(output.stdout.contains("Pinned budget:"));
    assert!(output.stdout.contains("README.md"));
    assert!(output.stdout.contains("required"));
    assert!(output.stdout.contains("Retrieval context: included"));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn project_context_select_optional_pin_text_reports_omission() {
    let temp = fixture();
    let output = render_cli_invocation(
        parse_cli_invocation([
            "repo",
            "context",
            "select",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
            "--task",
            "docs design",
            "--required-pin",
            "README.md",
            "--optional-pin",
            "docs/design.md",
            "--pin-reason",
            "design context",
            "--context-budget",
            "30",
            "--reserved-response-tokens",
            "10",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    assert!(
        output
            .stdout
            .contains("Project context pinned selection: ready")
    );
    assert!(output.stdout.contains("Omitted pins: 1"));
    assert!(output.stdout.contains("docs/design.md"));
    assert!(output.stdout.contains("budget_exceeded"));
}

#[test]
fn project_context_select_pinned_json_emits_snapshot_and_done() {
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
            "--required-pin",
            "README.md",
            "--pin-reason",
            "entry api_key",
            "--semantic",
            "fixture",
            "--context-budget",
            "400",
            "--reserved-response-tokens",
            "50",
        ])
        .expect("parse select"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ProjectContextDetail(ruse_ui_bridge::UiProjectContextDetail::Pinned(snapshot)))
            if snapshot.workspace_root_display == "."
                && snapshot.task == "command [redacted] palette"
                && snapshot.retrieval_kind == "hybrid"
                && snapshot.included.iter().any(|item| item.path == "README.md" && item.priority == "required")
                && snapshot.retrieval_included
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count == 0
    ));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
}

#[test]
fn project_context_select_pinned_invalid_inputs_exit_two() {
    let temp = fixture();
    for args in [
        vec![
            "repo",
            "context",
            "select",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
            "--task",
            "update CLI context",
            "--required-pin",
            "missing.rs",
        ],
        vec![
            "repo",
            "context",
            "select",
            "--task",
            "update CLI context",
            "--required-pin",
            "README.md",
            "--pin-reason",
            " ",
        ],
        vec![
            "repo",
            "context",
            "select",
            "--task",
            "update CLI context",
            "--context-budget",
            "10",
            "--reserved-response-tokens",
            "10",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
            .args(args)
            .output()
            .expect("ruse-cli binary should run");
        assert_eq!(output.status.code(), Some(2));
    }
}
