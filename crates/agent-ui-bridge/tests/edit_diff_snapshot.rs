use agent_core::{DiffEvidenceState, DiffSummary, EditDiffFlowReport, EditPreviewSummary};
use agent_types::{ConversationId, WorkspaceId};
use agent_ui_bridge::{UiEditDiffSnapshot, UiJsonFrame, encode_json_frame};

fn report() -> EditDiffFlowReport {
    EditDiffFlowReport {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        path: "README.md".to_string(),
        applied: true,
        preview: EditPreviewSummary {
            resource: "README.md".to_string(),
            summary: "Edit README.md: +1 -1 lines".to_string(),
            content_hash: Some("fnv1a64:edit".to_string()),
            truncated: false,
            redacted: false,
        },
        changed_files: vec!["README.md".to_string()],
        diff: DiffSummary {
            state: DiffEvidenceState::Applied,
            changed_file_count: 1,
            diff_line_count: 6,
            added_line_count: 1,
            removed_line_count: 1,
            preview: "--- README.md\n+++ README.md\n@@\n-old text\n+new text\n".to_string(),
            truncated: false,
            redacted: false,
        },
        events: Vec::new(),
        diagnostics: Vec::new(),
    }
}

#[test]
fn edit_diff_snapshot_serializes_safe_shape() {
    let snapshot = UiEditDiffSnapshot::from_report(&report()).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(value["path"], "README.md");
    assert_eq!(value["applied"], true);
    assert_eq!(value["diff_state"], "applied");
    assert_eq!(value["changed_files"][0], "README.md");
    assert_eq!(value["added_line_count"], 1);
    assert_eq!(value["removed_line_count"], 1);
    assert_eq!(value["resource"], "README.md");
    assert!(value["summary"].as_str().unwrap().contains("+1 -1"));
}

#[test]
fn edit_diff_snapshot_redacts_secret_like_preview() {
    let mut report = report();
    report.diff.preview = "+sk-test-secret\n+api_key = value\n".to_string();
    report.diff.redacted = true;

    let snapshot = UiEditDiffSnapshot::from_report(&report).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(serialized.contains("[redacted]"));
    assert!(snapshot.redacted);
}

#[test]
fn edit_diff_snapshot_json_frame_is_single_line() {
    let snapshot = UiEditDiffSnapshot::from_report(&report()).expect("snapshot");

    let line = encode_json_frame(&UiJsonFrame::EditDiffSnapshot(snapshot)).expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(frame, UiJsonFrame::EditDiffSnapshot(_)));
}
