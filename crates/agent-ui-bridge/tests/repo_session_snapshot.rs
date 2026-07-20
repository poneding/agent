use agent_core::{
    RepoConfigSummary, RepoGitState, RepoGitSummary, RepoProviderSummary,
    RepoSessionBootstrapReport, RepoSessionDiagnostic, RepoSessionDiagnosticSeverity,
};
use agent_types::{ConversationId, WorkspaceId, WorkspaceTrustLevel};
use agent_ui_bridge::{UiJsonFrame, UiRepoSessionSnapshot, encode_json_frame};

fn report() -> RepoSessionBootstrapReport {
    RepoSessionBootstrapReport {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: None,
        workspace_root_display: ".".to_string(),
        trust_level: WorkspaceTrustLevel::Untrusted,
        git: RepoGitSummary {
            state: RepoGitState::Dirty,
            branch: Some("main".to_string()),
            changed_file_count: 2,
            summary: "dirty; changed files: 2".to_string(),
        },
        config: RepoConfigSummary {
            schema_version: 1,
            selected_provider: "fake".to_string(),
            selected_model: "fake-scripted".to_string(),
            provider_ids: vec!["fake".to_string()],
            layer_count: 1,
            diagnostic_count: 0,
        },
        provider: RepoProviderSummary {
            selected_provider: "fake".to_string(),
            selected_model: "fake-scripted".to_string(),
            provider_kind: "fake".to_string(),
            provider_count: 1,
            enabled_provider_count: 1,
        },
        events: Vec::new(),
        diagnostics: vec![RepoSessionDiagnostic {
            severity: RepoSessionDiagnosticSeverity::Warning,
            code: "repo_session.git.dirty".to_string(),
            message: "workspace has uncommitted changes".to_string(),
            recoverable: true,
        }],
    }
}

#[test]
fn repo_session_snapshot_serializes_safe_shape() {
    let snapshot = UiRepoSessionSnapshot::from_report(&report()).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(value["workspace_root_display"], ".");
    assert_eq!(value["trust_level"], "untrusted");
    assert_eq!(value["git_state"], "dirty");
    assert_eq!(value["git_branch"], "main");
    assert_eq!(value["changed_file_count"], 2);
    assert_eq!(value["selected_provider"], "fake");
    assert_eq!(value["selected_model"], "fake-scripted");
    assert_eq!(value["diagnostic_count"], 1);
}

#[test]
fn repo_session_snapshot_json_frame_is_single_line() {
    let snapshot = UiRepoSessionSnapshot::from_report(&report()).expect("snapshot");

    let line = encode_json_frame(&UiJsonFrame::repo_session_snapshot(snapshot)).expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(frame, UiJsonFrame::repo_session_snapshot(_)));
}

#[test]
fn repo_session_snapshot_rejects_empty_display_root() {
    let mut report = report();
    report.workspace_root_display.clear();

    let error = UiRepoSessionSnapshot::from_report(&report).expect_err("empty root rejected");

    assert!(error.to_string().contains("workspace root"));
}
