use ruse_core::{
    CodingFinalReview, FinalReviewCommandResult, FinalReviewDiagnostic,
    FinalReviewDiagnosticSeverity,
};
use ruse_types::{ConversationId, WorkspaceId};
use ruse_ui_bridge::{UiFinalReviewSnapshot, UiJsonFrame, encode_json_frame};

fn review() -> CodingFinalReview {
    CodingFinalReview {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        summary: "Local coding workflow completed".to_string(),
        changed_files: vec!["README.md".to_string()],
        command_results: vec![FinalReviewCommandResult {
            command: "cargo test".to_string(),
            status: "succeeded".to_string(),
            classification: "none".to_string(),
            exit_code: Some(0),
            timed_out: false,
            summary: "cargo test exited 0".to_string(),
        }],
        trace_id: Some("trace-123".to_string()),
        trace_event_count: 31,
        audit_event_count: 30,
        replay_reconstructed: true,
        unresolved_risks: Vec::new(),
        next_actions: vec!["Review and commit the verified changes.".to_string()],
        diagnostics: Vec::new(),
    }
}

#[test]
fn final_review_snapshot_serializes_safe_shape() {
    let snapshot = UiFinalReviewSnapshot::from_review(&review()).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(value["summary"], "Local coding workflow completed");
    assert_eq!(value["changed_files"][0], "README.md");
    assert_eq!(value["command_results"][0]["command"], "cargo test");
    assert_eq!(value["command_results"][0]["status"], "succeeded");
    assert_eq!(value["trace_id"], "trace-123");
    assert_eq!(value["trace_event_count"], 31);
    assert_eq!(value["audit_event_count"], 30);
    assert_eq!(value["replay_reconstructed"], true);
    assert_eq!(
        value["next_actions"][0],
        "Review and commit the verified changes."
    );
}

#[test]
fn final_review_snapshot_redacts_secret_like_content() {
    let mut review = review();
    review.summary = "summary sk-test-secret".to_string();
    review.changed_files = vec!["api_key.md".to_string()];
    review.trace_id = Some("trace-secret-token".to_string());
    review.command_results[0].summary = "password sk-test-secret".to_string();
    review.unresolved_risks = vec!["token leaked".to_string()];
    review.diagnostics = vec![FinalReviewDiagnostic {
        severity: FinalReviewDiagnosticSeverity::Warning,
        code: "risk".to_string(),
        message: "api_key found".to_string(),
        recoverable: true,
    }];

    let snapshot = UiFinalReviewSnapshot::from_review(&review).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("trace-secret-token"));
    assert!(serialized.contains("[redacted]"));
    assert_eq!(snapshot.diagnostic_count, 1);
}

#[test]
fn final_review_snapshot_json_frame_is_single_line() {
    let snapshot = UiFinalReviewSnapshot::from_review(&review()).expect("snapshot");

    let line = encode_json_frame(&UiJsonFrame::final_review_snapshot(snapshot)).expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(frame, UiJsonFrame::final_review_snapshot(_)));
}
