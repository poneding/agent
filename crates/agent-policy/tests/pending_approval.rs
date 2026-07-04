use agent_policy::{
    ApprovalDecision, ApprovalLeaseScope, ApprovalPreviewSummary, ApprovalResponse,
    ApprovalResumeToken, ApprovalStatus, CapabilityAction, PendingApproval, PermissionRequest,
    ResourceKind, ResourceScope,
};
use agent_types::{ActorRef, ApprovalRequestId, CheckpointId, CorrelationId, Timestamp};
use serde_json::json;
use uuid::Uuid;

fn ts(second: u8) -> Timestamp {
    let value = match second {
        0 => "2026-06-27T10:00:00Z",
        1 => "2026-06-27T10:00:01Z",
        2 => "2026-06-27T10:00:02Z",
        10 => "2026-06-27T10:00:10Z",
        11 => "2026-06-27T10:00:11Z",
        20 => "2026-06-27T10:00:20Z",
        30 => "2026-06-27T10:00:30Z",
        31 => "2026-06-27T10:00:31Z",
        60 => "2026-06-27T10:01:00Z",
        61 => "2026-06-27T10:01:01Z",
        62 => "2026-06-27T10:01:02Z",
        _ => panic!("unexpected test timestamp offset"),
    };
    serde_json::from_value(json!(value)).expect("valid timestamp")
}

fn permission_request() -> PermissionRequest {
    PermissionRequest::new(
        ActorRef::system(),
        CapabilityAction::Execute,
        ResourceScope::new(ResourceKind::Command, "cargo test").expect("valid scope"),
        "run verification",
        CorrelationId::from_uuid(Uuid::from_u128(9)),
    )
    .expect("valid request")
}

fn pending_approval(max_duration_seconds: Option<u64>) -> PendingApproval {
    let request = permission_request();
    let scope = ApprovalLeaseScope::from_request(&request, max_duration_seconds);
    let preview = ApprovalPreviewSummary::new(
        "command",
        "cargo test",
        "Run cargo test inside the workspace",
    )
    .expect("valid preview")
    .with_content_hash("sha256:preview")
    .with_truncated(true)
    .with_redacted(true);
    let resume_token = ApprovalResumeToken::new(
        CheckpointId::from_uuid(Uuid::from_u128(7)),
        "before command execution",
    )
    .expect("valid resume token");

    PendingApproval::new_with_id(
        ApprovalRequestId::from_uuid(Uuid::from_u128(1)),
        request,
        preview,
        scope,
        Some(resume_token),
        ts(0),
        Some(ts(60)),
    )
    .expect("valid pending approval")
}

#[test]
fn pending_approval_serializes_stable_shape_without_raw_input() {
    let approval = pending_approval(Some(300));

    assert_eq!(
        serde_json::to_value(&approval).expect("approval json"),
        json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "permission_request": {
                "id": approval.permission_request.id,
                "actor": {
                    "kind": "system",
                    "id": "system",
                    "display_name": "system"
                },
                "action": "execute",
                "scope": {
                    "kind": "command",
                    "pattern": "cargo test"
                },
                "reason": "run verification",
                "correlation_id": "00000000-0000-0000-0000-000000000009"
            },
            "preview": {
                "kind": "command",
                "resource": "cargo test",
                "summary": "Run cargo test inside the workspace",
                "content_hash": "sha256:preview",
                "truncated": true,
                "redacted": true
            },
            "lease_scope": {
                "actor": {
                    "kind": "system",
                    "id": "system",
                    "display_name": "system"
                },
                "action": "execute",
                "resource": {
                    "kind": "command",
                    "pattern": "cargo test"
                },
                "max_duration_seconds": 300
            },
            "resume_token": {
                "checkpoint_id": "00000000-0000-0000-0000-000000000007",
                "label": "before command execution"
            },
            "status": "pending",
            "created_at": "2026-06-27T10:00:00Z",
            "expires_at": "2026-06-27T10:01:00Z",
            "resolved_at": null,
            "response": null
        })
    );
}

#[test]
fn approving_pending_approval_clamps_lease_duration_and_can_be_consumed() {
    let mut approval = pending_approval(Some(300));
    let response = ApprovalResponse::allow(
        approval.id,
        "approved for this verification",
        Some(600),
        ts(10),
    )
    .expect("valid approval response");

    let grant = approval.approve(response).expect("approval grant");

    assert_eq!(approval.status, ApprovalStatus::Approved);
    assert_eq!(approval.resolved_at, Some(ts(10)));
    assert_eq!(
        approval.response.as_ref().map(|r| r.decision),
        Some(ApprovalDecision::Allow)
    );
    assert_eq!(grant.request_id, approval.id);
    assert_eq!(grant.duration_seconds, Some(300));
    assert_eq!(grant.scope, approval.lease_scope);

    approval.consume(ts(20)).expect("consume approved approval");

    assert_eq!(approval.status, ApprovalStatus::Consumed);
    assert_eq!(approval.resolved_at, Some(ts(20)));
}

#[test]
fn denying_pending_approval_stores_reason_and_does_not_grant_lease() {
    let mut approval = pending_approval(Some(300));
    let response = ApprovalResponse::deny(approval.id, "command is too broad", ts(11))
        .expect("valid denial response");

    approval.deny(response).expect("deny approval");

    assert_eq!(approval.status, ApprovalStatus::Denied);
    assert_eq!(approval.resolved_at, Some(ts(11)));
    assert_eq!(
        approval.response.as_ref().map(|r| r.reason.as_str()),
        Some("command is too broad")
    );
}

#[test]
fn stale_expired_and_cancelled_approvals_reject_responses() {
    let mut expired = pending_approval(Some(300));
    expired.expire(ts(61)).expect("expire approval");
    assert_eq!(expired.status, ApprovalStatus::Expired);
    assert!(
        expired
            .approve(ApprovalResponse::allow(expired.id, "late", Some(10), ts(62)).unwrap())
            .is_err()
    );

    let mut cancelled = pending_approval(Some(300));
    cancelled.cancel(ts(30)).expect("cancel approval");
    assert_eq!(cancelled.status, ApprovalStatus::Cancelled);
    assert!(
        cancelled
            .deny(ApprovalResponse::deny(cancelled.id, "too late", ts(31)).unwrap())
            .is_err()
    );
}

#[test]
fn approval_contracts_reject_invalid_summaries_and_mismatched_responses() {
    assert!(ApprovalPreviewSummary::new("command", "", "summary").is_err());
    assert!(ApprovalResumeToken::new(CheckpointId::new(), " ").is_err());
    assert!(ApprovalResponse::deny(ApprovalRequestId::new(), " ", ts(1)).is_err());

    let mut approval = pending_approval(Some(300));
    let mismatched = ApprovalResponse::allow(ApprovalRequestId::new(), "wrong id", None, ts(2))
        .expect("valid shape");

    assert!(approval.approve(mismatched).is_err());
}
