use ruse_desktop::{
    DesktopApprovalReview, DesktopApprovalReviewStatus, DesktopDiffFileReview,
    DesktopDiffHunkPreview, DesktopDiffLineKind, DesktopDiffLinePreview, DesktopDiffReview,
    DesktopError, DesktopPermissionDecision, DiffFileSummary, DiffReview, DiffReviewStatus,
    PermissionDialog, PermissionDialogStatus, approve_diff_review, expire_permission_dialog,
    reject_diff_review, resolve_permission_dialog,
};
use ruse_policy::{CapabilityAction, ResourceKind, ResourceScope};
use ruse_types::{TaskId, Timestamp};
use ruse_ui_bridge::{UiApprovalPreviewSummary, UiApprovalRequest};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-25T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000000001"
        .parse()
        .expect("valid task id")
}

fn scope() -> ResourceScope {
    ResourceScope::new(ResourceKind::File, "src/**").expect("valid scope")
}

fn permission_dialog() -> PermissionDialog {
    PermissionDialog::new(
        "permission-1",
        CapabilityAction::Read,
        scope(),
        "Read source files",
        ts(1),
    )
    .expect("valid permission dialog")
}

fn diff_file(path: &str) -> DiffFileSummary {
    DiffFileSummary::new(path, 10, 2, "Updates desktop contracts").expect("valid diff summary")
}

fn diff_review() -> DiffReview {
    DiffReview::new(
        task_id(),
        vec![diff_file("crates/ruse-desktop/src/lib.rs")],
        ts(1),
    )
    .expect("valid diff review")
}

fn approval_request(status: &str, summary: &str) -> UiApprovalRequest {
    UiApprovalRequest {
        request_id: "approval-1".to_string(),
        status: status.to_string(),
        action: "execute".to_string(),
        resource_kind: "command".to_string(),
        resource: "cargo test".to_string(),
        reason: "run requested verification".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "command".to_string(),
            resource: "cargo test".to_string(),
            summary: summary.to_string(),
            content_hash: Some("hash-1".to_string()),
            truncated: false,
            redacted: false,
        },
        max_duration_seconds: Some(120),
        expires_at: Some(ts(9)),
    }
}

#[test]
fn permissions_diffs_serializes_permission_dialog_shape() {
    let dialog = permission_dialog();

    let value = serde_json::to_value(dialog).expect("serialize permission dialog");

    assert_eq!(value["request_id"], "permission-1");
    assert_eq!(value["action"], "read");
    assert_eq!(value["scope"]["kind"], "file");
    assert_eq!(value["scope"]["pattern"], "src/**");
    assert_eq!(value["reason"], "Read source files");
    assert_eq!(value["status"], "pending");
    assert_eq!(value["created_at"], "2026-06-25T00:00:01Z");
    assert!(value["decided_at"].is_null());
}

#[test]
fn permissions_diffs_transitions_permission_decisions() {
    let allowed = resolve_permission_dialog(
        &permission_dialog(),
        DesktopPermissionDecision::AllowOnce,
        ts(2),
    )
    .unwrap();
    let denied =
        resolve_permission_dialog(&permission_dialog(), DesktopPermissionDecision::Deny, ts(3))
            .unwrap();
    let expired = expire_permission_dialog(&permission_dialog(), ts(4)).unwrap();

    assert_eq!(allowed.status, PermissionDialogStatus::Allowed);
    assert_eq!(allowed.decided_at, Some(ts(2)));
    assert_eq!(denied.status, PermissionDialogStatus::Denied);
    assert_eq!(denied.decided_at, Some(ts(3)));
    assert_eq!(expired.status, PermissionDialogStatus::Expired);
    assert_eq!(expired.decided_at, Some(ts(4)));
}

#[test]
fn permissions_diffs_rejects_terminal_permission_transition() {
    let allowed = resolve_permission_dialog(
        &permission_dialog(),
        DesktopPermissionDecision::AllowOnce,
        ts(2),
    )
    .unwrap();

    let err =
        resolve_permission_dialog(&allowed, DesktopPermissionDecision::Deny, ts(3)).unwrap_err();

    assert!(matches!(err, DesktopError::InvalidPermission(_)));
}

#[test]
fn permissions_diffs_builds_desktop_approval_review_from_bridge_request() {
    let review = DesktopApprovalReview::from_ui_request(&approval_request(
        "pending",
        "run token sk-secret with password",
    ))
    .unwrap();
    let serialized = serde_json::to_string(&review).expect("serialize approval review");

    assert_eq!(review.status, DesktopApprovalReviewStatus::Pending);
    assert!(review.decision_enabled);
    assert_eq!(review.lease_duration_seconds, Some(120));
    assert!(review.disabled_reason.is_none());
    assert!(review.preview.summary.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
}

#[test]
fn permissions_diffs_disables_terminal_desktop_approval_reviews() {
    let allowed = DesktopApprovalReview::from_ui_request(&approval_request("allowed", "ok"))
        .expect("allowed review");
    let expired = DesktopApprovalReview::from_ui_request(&approval_request("expired", "ok"))
        .expect("expired review");

    assert_eq!(allowed.status, DesktopApprovalReviewStatus::Allowed);
    assert!(!allowed.decision_enabled);
    assert_eq!(
        allowed.disabled_reason.as_deref(),
        Some("approval already allowed")
    );
    assert_eq!(expired.status, DesktopApprovalReviewStatus::Expired);
    assert!(!expired.decision_enabled);
    assert_eq!(expired.disabled_reason.as_deref(), Some("approval expired"));
}

#[test]
fn permissions_diffs_serializes_diff_review_shape() {
    let review = approve_diff_review(&diff_review(), ts(2)).unwrap();

    let value = serde_json::to_value(review).expect("serialize diff review");

    assert_eq!(value["task_id"], "00000000-0000-0000-0000-000000000001");
    assert_eq!(value["files"][0]["path"], "crates/ruse-desktop/src/lib.rs");
    assert_eq!(value["files"][0]["added_lines"], 10);
    assert_eq!(value["files"][0]["removed_lines"], 2);
    assert_eq!(value["files"][0]["summary"], "Updates desktop contracts");
    assert_eq!(value["status"], "approved");
    assert_eq!(value["decided_at"], "2026-06-25T00:00:02Z");
}

#[test]
fn permissions_diffs_rejects_unsafe_diff_paths() {
    let parent_traversal = DiffFileSummary::new("../secret", 1, 0, "bad path");
    let absolute = DiffFileSummary::new("/tmp/secret", 1, 0, "bad path");
    let empty_summary = DiffFileSummary::new("src/lib.rs", 1, 0, " ");
    let rejected = reject_diff_review(&diff_review(), ts(2)).unwrap();
    let terminal = approve_diff_review(&rejected, ts(3)).unwrap_err();

    assert!(matches!(
        parent_traversal,
        Err(DesktopError::InvalidDiff(_))
    ));
    assert!(matches!(absolute, Err(DesktopError::InvalidDiff(_))));
    assert!(matches!(empty_summary, Err(DesktopError::InvalidDiff(_))));
    assert!(matches!(terminal, DesktopError::InvalidDiff(_)));
}

#[test]
fn permissions_diffs_builds_visual_diff_review_with_bounded_redacted_hunks() {
    let line = DesktopDiffLinePreview::new(
        DesktopDiffLineKind::Addition,
        None,
        Some(10),
        format!("+ token sk-secret password {}", "x".repeat(300)),
    )
    .expect("valid diff line");
    let hunk = DesktopDiffHunkPreview::new(8, 10, vec![line]).expect("valid hunk");
    let file = DesktopDiffFileReview::new("src/main.rs", 1, 0, "adds tested branch", vec![hunk])
        .expect("valid file review");
    let review = DesktopDiffReview::new(
        "review-1",
        DiffReviewStatus::Pending,
        vec![file],
        Some("src/main.rs".to_string()),
    )
    .expect("valid desktop diff review");
    let serialized = serde_json::to_string(&review).expect("serialize desktop diff review");

    assert!(review.decision_enabled);
    assert_eq!(review.selected_path.as_deref(), Some("src/main.rs"));
    assert!(review.files[0].hunks[0].truncated);
    assert!(
        review.files[0].hunks[0].lines[0]
            .content
            .contains("[redacted]")
    );
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
}

#[test]
fn permissions_diffs_disables_terminal_visual_diff_reviews() {
    let line = DesktopDiffLinePreview::new(DesktopDiffLineKind::Context, Some(1), Some(1), "same")
        .expect("valid line");
    let hunk = DesktopDiffHunkPreview::new(1, 1, vec![line]).expect("valid hunk");
    let file = DesktopDiffFileReview::new("src/lib.rs", 0, 0, "no changes", vec![hunk])
        .expect("valid file review");
    let review = DesktopDiffReview::new(
        "review-1",
        DiffReviewStatus::Rejected,
        vec![file],
        Some("src/lib.rs".to_string()),
    )
    .expect("valid rejected review");
    let missing_selected_path = DesktopDiffReview::new(
        "review-2",
        DiffReviewStatus::Pending,
        vec![
            DesktopDiffFileReview::new(
                "src/lib.rs",
                0,
                0,
                "no changes",
                vec![
                    DesktopDiffHunkPreview::new(
                        1,
                        1,
                        vec![
                            DesktopDiffLinePreview::new(
                                DesktopDiffLineKind::Context,
                                Some(1),
                                Some(1),
                                "same",
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
        Some("src/main.rs".to_string()),
    );

    assert!(!review.decision_enabled);
    assert_eq!(
        review.disabled_reason.as_deref(),
        Some("diff review already rejected")
    );
    assert!(matches!(
        missing_selected_path,
        Err(DesktopError::InvalidDiff(_))
    ));
}
