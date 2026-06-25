use agent_desktop::{
    DesktopError, DesktopPermissionDecision, DiffFileSummary, DiffReview, PermissionDialog,
    PermissionDialogStatus, approve_diff_review, expire_permission_dialog, reject_diff_review,
    resolve_permission_dialog,
};
use agent_policy::{CapabilityAction, ResourceKind, ResourceScope};
use agent_types::{TaskId, Timestamp};

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
        vec![diff_file("crates/agent-desktop/src/lib.rs")],
        ts(1),
    )
    .expect("valid diff review")
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
fn permissions_diffs_serializes_diff_review_shape() {
    let review = approve_diff_review(&diff_review(), ts(2)).unwrap();

    let value = serde_json::to_value(review).expect("serialize diff review");

    assert_eq!(value["task_id"], "00000000-0000-0000-0000-000000000001");
    assert_eq!(value["files"][0]["path"], "crates/agent-desktop/src/lib.rs");
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
