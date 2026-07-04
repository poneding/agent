use std::fs;
use std::process::Command;

use agent_core::{RepoGitState, RepoSessionBootstrapInput, bootstrap_repo_session};
use agent_types::{ClientKind, RuntimeEventPayload};

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn git(repo: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git command runs");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn input(root: &std::path::Path) -> RepoSessionBootstrapInput {
    RepoSessionBootstrapInput {
        workspace_root: root.to_path_buf(),
        config_path: None,
        provider_override: None,
        model_override: None,
        client_kind: ClientKind::Cli,
        client_version: "test".to_string(),
    }
}

#[test]
fn repo_session_bootstrap_git_workspace_emits_session_events() {
    if !git_available() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    git(temp.path(), &["init"]);

    let report = bootstrap_repo_session(input(temp.path())).expect("bootstrap repo session");

    assert_eq!(report.workspace_root_display, ".");
    assert_eq!(report.git.state, RepoGitState::Clean);
    assert_eq!(report.git.changed_file_count, 0);
    assert_eq!(report.events.len(), 3);
    assert!(matches!(
        report.events[0].payload,
        RuntimeEventPayload::WorkspaceOpened(_)
    ));
    assert!(matches!(
        report.events[1].payload,
        RuntimeEventPayload::SessionStarted(_)
    ));
    assert!(matches!(
        report.events[2].payload,
        RuntimeEventPayload::ConversationStarted(_)
    ));
    assert!(report.events.iter().all(|event| {
        event.workspace_id == report.workspace_id
            && event.conversation_id == report.conversation_id
            && event.turn_id.is_none()
    }));
    assert_eq!(report.provider.selected_provider, "fake");
    assert_eq!(report.provider.selected_model, "fake-scripted");
}

#[test]
fn repo_session_bootstrap_dirty_git_workspace_reports_dirty_without_paths() {
    if !git_available() {
        return;
    }
    let temp = tempfile::tempdir().expect("tempdir");
    git(temp.path(), &["init"]);
    fs::write(temp.path().join("token.txt"), "not a real secret").expect("write dirty file");

    let report = bootstrap_repo_session(input(temp.path())).expect("bootstrap dirty repo");

    assert_eq!(report.git.state, RepoGitState::Dirty);
    assert_eq!(report.git.changed_file_count, 1);
    assert!(!report.git.summary.contains("token.txt"));
}

#[test]
fn repo_session_bootstrap_non_git_workspace_succeeds_with_warning() {
    let temp = tempfile::tempdir().expect("tempdir");

    let report = bootstrap_repo_session(input(temp.path())).expect("bootstrap non-git dir");

    assert_eq!(report.git.state, RepoGitState::NotRepository);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "repo_session.git.not_repository" && diagnostic.recoverable
    }));
    assert_eq!(report.events.len(), 3);
}

#[test]
fn repo_session_bootstrap_invalid_workspace_fails_before_events() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");

    let error = bootstrap_repo_session(input(&missing)).expect_err("missing workspace rejected");

    assert_eq!(error.code(), "repo_session.workspace.invalid");
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == "repo_session.workspace.invalid")
    );
}

#[test]
fn repo_session_bootstrap_config_errors_are_redacted() {
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
    let mut request = input(temp.path());
    request.config_path = Some(config_path);

    let error = bootstrap_repo_session(request).expect_err("invalid config rejected");
    let rendered = format!("{error:?}");

    assert_eq!(error.code(), "repo_session.config.invalid");
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("OPENAI_API_KEY"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("secret"));
}
