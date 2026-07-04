use std::fs;
use std::path::Path;
use std::process::Command;

use agent_core::{
    DiffEvidenceState, EditDiffFlowError, EditDiffFlowInput, RuntimeIds, run_edit_diff_flow,
};
use agent_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

fn input(root: &Path, apply: bool) -> EditDiffFlowInput {
    EditDiffFlowInput {
        workspace_root: root.to_path_buf(),
        path: "README.md".to_string(),
        expected: Some("old text\n".to_string()),
        replacement: "new text\n".to_string(),
        apply,
        ids: runtime_ids(),
    }
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn workspace(git_repo: bool) -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("README.md"), "old text\n").expect("write readme");
    if git_repo {
        git(temp.path(), &["init", "-q"]);
        git(temp.path(), &["config", "user.name", "Agent"]);
        git(
            temp.path(),
            &["config", "user.email", "agent@example.invalid"],
        );
        git(temp.path(), &["add", "README.md"]);
        git(temp.path(), &["commit", "-m", "initial", "--quiet"]);
    }
    temp
}

#[test]
fn edit_diff_flow_preview_does_not_mutate_file() {
    if !git_available() {
        return;
    }
    let temp = workspace(true);

    let report = run_edit_diff_flow(input(temp.path(), false)).expect("preview edit");

    assert!(!report.applied);
    assert_eq!(report.path, "README.md");
    assert_eq!(report.changed_files, Vec::<String>::new());
    assert_eq!(report.diff.state, DiffEvidenceState::Preview);
    assert!(report.preview.summary.contains("+1 -1"));
    assert!(report.diff.preview.contains("-old text"));
    assert!(report.diff.preview.contains("+new text"));
    assert_eq!(
        fs::read_to_string(temp.path().join("README.md")).expect("read file"),
        "old text\n"
    );
}

#[test]
fn edit_diff_flow_apply_mutates_and_reports_git_diff() {
    if !git_available() {
        return;
    }
    let temp = workspace(true);

    let report = run_edit_diff_flow(input(temp.path(), true)).expect("apply edit");

    assert!(report.applied);
    assert_eq!(
        fs::read_to_string(temp.path().join("README.md")).expect("read file"),
        "new text\n"
    );
    assert_eq!(report.changed_files, vec!["README.md"]);
    assert_eq!(report.diff.state, DiffEvidenceState::Applied);
    assert_eq!(report.diff.changed_file_count, 1);
    assert_eq!(report.diff.added_line_count, 1);
    assert_eq!(report.diff.removed_line_count, 1);
    assert!(report.diff.preview.contains("-old text"));
    assert!(report.diff.preview.contains("+new text"));
}

#[test]
fn edit_diff_flow_expected_mismatch_leaves_file_unchanged() {
    let temp = workspace(false);
    let mut request = input(temp.path(), true);
    request.expected = Some("different\n".to_string());

    let error = run_edit_diff_flow(request).expect_err("mismatch rejected");

    assert_eq!(error.code(), "edit_diff.tool_failed");
    assert_eq!(
        fs::read_to_string(temp.path().join("README.md")).expect("read file"),
        "old text\n"
    );
}

#[test]
fn edit_diff_flow_non_git_apply_succeeds_with_recoverable_diagnostic() {
    let temp = workspace(false);

    let report = run_edit_diff_flow(input(temp.path(), true)).expect("apply non-git edit");

    assert!(report.applied);
    assert_eq!(report.changed_files, vec!["README.md"]);
    assert_eq!(report.diff.state, DiffEvidenceState::Unavailable);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "edit_diff.git_diff.unavailable" && diagnostic.recoverable
    }));
}

#[test]
fn edit_diff_flow_rejects_traversal_before_mutation() {
    let temp = workspace(false);
    fs::write(temp.path().join("outside.txt"), "outside\n").expect("write outside");
    let mut request = input(temp.path(), true);
    request.path = "../outside.txt".to_string();
    request.expected = Some("outside\n".to_string());
    request.replacement = "changed\n".to_string();

    let error = run_edit_diff_flow(request).expect_err("traversal rejected");

    assert!(matches!(error, EditDiffFlowError::ToolFailed { .. }));
    assert_eq!(
        fs::read_to_string(temp.path().join("outside.txt")).expect("read outside"),
        "outside\n"
    );
}

#[test]
fn edit_diff_flow_returns_tool_loop_events_in_order() {
    if !git_available() {
        return;
    }
    let temp = workspace(true);

    let report = run_edit_diff_flow(input(temp.path(), true)).expect("apply edit");

    let phases: Vec<&'static str> = report
        .events
        .iter()
        .filter_map(|event| match &event.payload {
            RuntimeEventPayload::ToolCallProposed(call)
                if call.tool_name == "workspace.edit_file" =>
            {
                Some("proposed")
            }
            RuntimeEventPayload::PermissionDecisionRecorded(_) => Some("decision"),
            RuntimeEventPayload::ToolExecutionStarted(started)
                if started.tool_name == "workspace.edit_file" =>
            {
                Some("started")
            }
            RuntimeEventPayload::ToolExecutionCompleted(completed)
                if completed.call_id == "edit-diff-edit" =>
            {
                Some("completed")
            }
            RuntimeEventPayload::ToolObservationRecorded(observed)
                if observed.call_id == "edit-diff-edit" =>
            {
                Some("observed")
            }
            _ => None,
        })
        .collect();

    assert!(
        phases.windows(5).any(|window| {
            window == ["proposed", "decision", "started", "completed", "observed"]
        }),
        "phases were {phases:?}"
    );
}
