use agent_tui::{DiffPanelState, TuiApp, render_app_to_string};
use agent_ui_bridge::{UiApprovalPreviewSummary, UiApprovalRequest};

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
