use agent_tui::{DiffPanelAction, DiffPanelState, TuiAction, TuiApp, render_app_to_string};
use agent_ui_bridge::{
    UiApprovalPreviewSummary, UiApprovalRequest, UiEditDiffSnapshot, UiJsonFrame,
};

fn file_edit_request(truncated: bool) -> UiApprovalRequest {
    UiApprovalRequest {
        request_id: "00000000-0000-0000-0000-000000000001".to_string(),
        status: "pending".to_string(),
        action: "write".to_string(),
        resource_kind: "file".to_string(),
        resource: "src/lib.rs".to_string(),
        reason: "apply proposed edit".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "file_edit".to_string(),
            resource: "src/lib.rs".to_string(),
            summary: "Edit src/lib.rs: +12 -3 lines\n+pub fn run_agent() {}\n-secret".to_string(),
            content_hash: Some("sha256:file-edit".to_string()),
            truncated,
            redacted: true,
        },
        max_duration_seconds: None,
        expires_at: None,
    }
}

#[test]
fn diff_panel_builds_from_file_edit_approval_preview() {
    let diff = DiffPanelState::from_approval(&file_edit_request(false), 200);

    assert_eq!(diff.resource, "src/lib.rs");
    assert!(diff.summary.contains("+12 -3"));
    assert!(diff.content_hash.as_deref() == Some("sha256:file-edit"));
    assert!(diff.redacted);
    assert!(!diff.truncated);
}

#[test]
fn diff_panel_truncates_large_previews_and_reports_truncation() {
    let mut request = file_edit_request(false);
    request.preview.summary = "x".repeat(120);

    let diff = DiffPanelState::from_approval(&request, 24);

    assert_eq!(diff.summary.len(), 24);
    assert!(diff.truncated);
}

#[test]
fn diff_panel_ignores_non_file_edit_previews() {
    let mut request = file_edit_request(false);
    request.preview.kind = "command".to_string();

    assert!(DiffPanelState::maybe_from_approval(&request, 200).is_none());
}

#[test]
fn diff_panel_builds_from_edit_diff_snapshot() {
    let snapshot = UiEditDiffSnapshot {
        workspace_id: agent_types::WorkspaceId::new(),
        conversation_id: agent_types::ConversationId::new(),
        path: "README.md".to_string(),
        applied: true,
        diff_state: "applied".to_string(),
        changed_files: vec!["README.md".to_string()],
        changed_file_count: 1,
        diff_line_count: 5,
        added_line_count: 1,
        removed_line_count: 1,
        resource: "README.md".to_string(),
        summary: "Edit README.md: +1 -1 lines".to_string(),
        content_hash: Some("fnv1a64:edit".to_string()),
        diff_preview: "--- README.md\n+++ README.md\n@@\n-old\n+new".to_string(),
        truncated: false,
        redacted: false,
        diagnostic_count: 0,
    };

    let diff = DiffPanelState::from_edit_diff_snapshot(&snapshot, 200);

    assert_eq!(diff.resource, "README.md");
    assert_eq!(diff.diff_state, "applied");
    assert_eq!(diff.changed_file_count, 1);
    assert_eq!(diff.added_line_count, 1);
    assert_eq!(diff.removed_line_count, 1);
    assert_eq!(diff.focused_file_index, Some(0));
    assert!(diff.summary.contains("-old"));
    assert!(diff.summary.contains("+new"));
    assert_eq!(diff.content_hash.as_deref(), Some("fnv1a64:edit"));
}

#[test]
fn diff_panel_navigates_changed_files_and_preview_lines() {
    let snapshot = UiEditDiffSnapshot {
        workspace_id: agent_types::WorkspaceId::new(),
        conversation_id: agent_types::ConversationId::new(),
        path: "src/lib.rs".to_string(),
        applied: false,
        diff_state: "preview".to_string(),
        changed_files: vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
        changed_file_count: 2,
        diff_line_count: 4,
        added_line_count: 2,
        removed_line_count: 1,
        resource: "src/lib.rs".to_string(),
        summary: "Edit two files".to_string(),
        content_hash: None,
        diff_preview: "line 1\nline 2\nline 3".to_string(),
        truncated: false,
        redacted: false,
        diagnostic_count: 0,
    };
    let mut diff = DiffPanelState::from_edit_diff_snapshot(&snapshot, 200);

    diff.reduce(DiffPanelAction::NextFile);
    assert_eq!(diff.focused_file_index, Some(1));
    diff.reduce(DiffPanelAction::PreviousFile);
    assert_eq!(diff.focused_file_index, Some(0));
    diff.reduce(DiffPanelAction::ScrollDown);
    assert_eq!(diff.scroll_line, 1);
    assert!(!diff.render_text().contains("line 1"));
    assert!(diff.render_text().contains("line 2"));
    diff.reduce(DiffPanelAction::ScrollUp);
    assert_eq!(diff.scroll_line, 0);
}

#[test]
fn app_reducer_populates_diff_panel_from_edit_diff_snapshot() {
    let mut app = TuiApp::default();
    app.reduce(TuiAction::BridgeFrame(Box::new(
        UiJsonFrame::EditDiffSnapshot(UiEditDiffSnapshot {
            workspace_id: agent_types::WorkspaceId::new(),
            conversation_id: agent_types::ConversationId::new(),
            path: "README.md".to_string(),
            applied: false,
            diff_state: "preview".to_string(),
            changed_files: vec!["README.md".to_string()],
            changed_file_count: 1,
            diff_line_count: 5,
            added_line_count: 1,
            removed_line_count: 1,
            resource: "README.md".to_string(),
            summary: "Edit README.md".to_string(),
            content_hash: None,
            diff_preview: "-old\n+new".to_string(),
            truncated: false,
            redacted: false,
            diagnostic_count: 0,
        }),
    )));

    let rendered = render_app_to_string(&app, 100, 20).expect("render app");

    assert!(rendered.contains("Diff"));
    assert!(rendered.contains("preview"));
    assert!(rendered.contains("Files: 1/1"));
    assert!(rendered.contains("+1 -1"));
}

#[test]
fn diff_panel_renders_truncated_redacted_preview_in_side_panel() {
    let app = TuiApp {
        diff_panel: Some(DiffPanelState::from_approval(&file_edit_request(true), 80)),
        ..Default::default()
    };

    let rendered = render_app_to_string(&app, 96, 18).expect("render app");

    assert!(rendered.contains("Diff"));
    assert!(rendered.contains("src/lib.rs"));
    assert!(rendered.contains("sha256:file-edit"));
    assert!(rendered.contains("truncated"));
    assert!(rendered.contains("redacted"));
}
