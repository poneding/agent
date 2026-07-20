use std::fs;
use std::path::PathBuf;
use std::process::Command;

use ruse_cli::{
    CliCommand, OutputMode, RepoSessionCommand, parse_cli_invocation, render_cli_invocation,
};
use ruse_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

fn fake_config_path() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ruse-cli-repo-session-fake-config-{}.yaml",
        std::process::id()
    ));
    fs::write(
        &path,
        r#"
schemaVersion: 1
model:
  defaultProvider: fake
  invocation:
    timeoutMs: 30000
    maxAttempts: 1
    retryableFailures: []
    fallbackProviderIds: []
  providers:
    - id: fake
      kind: fake
      defaultModel: fake-scripted
      enabled: true
      timeoutMs: 1000
"#,
    )
    .expect("write fake config");
    path
}

#[test]
fn repo_session_bootstrap_parses_text_and_json_forms() {
    let text =
        parse_cli_invocation(["repo", "session", "--workspace", "."]).expect("parse repo session");
    assert_eq!(text.output_mode, OutputMode::Text);
    assert!(matches!(
        text.command,
        CliCommand::RepoSession(RepoSessionCommand::Bootstrap { .. })
    ));

    let json = parse_cli_invocation(["--json", "repo", "session", "--workspace", "."])
        .expect("parse json repo session");
    assert_eq!(json.output_mode, OutputMode::Json);
    assert!(matches!(
        json.command,
        CliCommand::RepoSession(RepoSessionCommand::Bootstrap { .. })
    ));
}

#[test]
fn repo_session_bootstrap_text_renders_safe_summary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut invocation = parse_cli_invocation([
        "repo",
        "session",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
    ])
    .expect("parse repo session");
    invocation.config_path = Some(fake_config_path());

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Repo session: ready"));
    assert!(output.stdout.contains("Workspace: ."));
    assert!(output.stdout.contains("Git: not_repository"));
    assert!(output.stdout.contains("Provider: fake / fake-scripted"));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
}

#[test]
fn repo_session_bootstrap_json_renders_snapshot_and_events() {
    let temp = tempfile::tempdir().expect("tempdir");
    let mut invocation = parse_cli_invocation([
        "--json",
        "repo",
        "session",
        "--workspace",
        temp.path().to_str().expect("utf8 path"),
    ])
    .expect("parse repo session");
    invocation.config_path = Some(fake_config_path());

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::repo_session_snapshot(snapshot))
            if snapshot.workspace_root_display == "."
                && snapshot.git_state == "not_repository"
                && snapshot.selected_provider == "fake"
    ));
    let event_count = frames
        .iter()
        .filter(|frame| matches!(frame, UiJsonFrame::Event(_)))
        .count();
    assert_eq!(event_count, 3);
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count: done_count }) if *done_count == 3
    ));
    assert!(
        !output
            .stdout
            .contains(temp.path().to_str().expect("utf8 path"))
    );
}

#[test]
fn repo_session_bootstrap_invalid_workspace_exits_two() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");

    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .args([
            "repo",
            "session",
            "--workspace",
            missing.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("ruse-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn repo_session_bootstrap_json_redacts_config_secret_names() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config_path = temp.path().join("agent.toml");
    fs::write(
        &config_path,
        r#"
schema_version = 1

[model]
default_provider = "openai"

[model.invocation]
timeout_ms = 30000
max_attempts = 1
retryable_failures = []
fallback_provider_ids = []

[[model.providers]]
id = "openai"
kind = "open_ai_compatible"
default_model = "gpt-4.1-mini"
enabled = true
api_key_provider = "env"
api_key_name = "OPENAI_API_KEY"
api_key_value = "sk-test-secret"
"#,
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .args([
            "--json",
            "--config",
            config_path.to_str().expect("utf8 path"),
            "repo",
            "session",
            "--workspace",
            temp.path().to_str().expect("utf8 path"),
        ])
        .output()
        .expect("ruse-cli binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("sk-test-secret"));
    assert!(!stderr.contains("sk-test-secret"));
    assert!(!stdout.contains("OPENAI_API_KEY"));
    assert!(!stderr.contains("OPENAI_API_KEY"));
    assert!(!stdout.contains("api_key"));
    assert!(!stderr.contains("api_key"));
    assert!(!stdout.contains("secret"));
    assert!(!stderr.contains("secret"));
}
