use ruse_policy::{
    ApprovalLeaseScope, ApprovalPreviewSummary, ApprovalResponse, ApprovalResumeToken,
    CapabilityAction, PendingApproval, PermissionRequest, ResourceKind, ResourceScope,
};
use ruse_types::{
    ActorRef, ApprovalRequestId, CheckpointId, ClientKind, ConversationId, CorrelationId, EventId,
    SchemaVersion, Timestamp,
};
use ruse_ui_bridge::{
    UiApprovalDecision, UiApprovalRequest, UiClientInfo, UiCommand, UiCommandEnvelope,
    UiEventEnvelope, UiEventPayload, UiJsonFrame, validate_ui_command,
};
use serde_json::json;
use time::macros::datetime;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn timestamp() -> Timestamp {
    Timestamp::from_utc(datetime!(2026-06-27 10:00:00 UTC))
}

fn pending_approval() -> PendingApproval {
    let permission_request = PermissionRequest::new(
        ActorRef::system(),
        CapabilityAction::Execute,
        ResourceScope::new(ResourceKind::Command, "cargo test").expect("valid scope"),
        "run verification",
        id::<CorrelationId>("00000000-0000-0000-0000-000000000009"),
    )
    .expect("valid permission request");
    let lease_scope = ApprovalLeaseScope::from_request(&permission_request, Some(300));
    let preview = ApprovalPreviewSummary::new(
        "command",
        "cargo test",
        "Run cargo test inside the workspace",
    )
    .expect("valid preview")
    .with_content_hash("sha256:preview")
    .with_redacted(true);
    let resume_token = ApprovalResumeToken::new(
        id::<CheckpointId>("00000000-0000-0000-0000-000000000007"),
        "before command execution",
    )
    .expect("valid resume token");

    PendingApproval::new_with_id(
        id::<ApprovalRequestId>("00000000-0000-0000-0000-000000000001"),
        permission_request,
        preview,
        lease_scope,
        Some(resume_token),
        timestamp(),
        None,
    )
    .expect("valid pending approval")
}

fn event_envelope(payload: UiEventPayload) -> UiEventEnvelope {
    UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: id::<EventId>("00000000-0000-0000-0000-000000000005"),
        seq: None,
        occurred_at: timestamp(),
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
        turn_id: None,
        task_id: None,
        correlation_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000004"),
        payload,
    }
}

fn command_envelope(command: UiCommand) -> UiCommandEnvelope {
    UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000100"),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command,
    }
}

#[test]
fn approval_requested_frame_serializes_safe_preview() {
    let request = UiApprovalRequest::from_pending_approval(&pending_approval());
    let frame = UiJsonFrame::Event(event_envelope(UiEventPayload::ApprovalRequested(Box::new(
        request,
    ))));

    let value = serde_json::to_value(frame).expect("serialize frame");

    assert_eq!(
        value,
        json!({
            "type": "event",
            "data": {
                "schema_version": 1,
                "event_id": "00000000-0000-0000-0000-000000000005",
                "seq": null,
                "occurred_at": "2026-06-27T10:00:00Z",
                "conversation_id": "00000000-0000-0000-0000-000000000002",
                "turn_id": null,
                "task_id": null,
                "correlation_id": "00000000-0000-0000-0000-000000000004",
                "payload": {
                    "type": "approval_requested",
                    "data": {
                        "request_id": "00000000-0000-0000-0000-000000000001",
                        "status": "pending",
                        "action": "execute",
                        "resource_kind": "command",
                        "resource": "cargo test",
                        "reason": "run verification",
                        "preview": {
                            "kind": "command",
                            "resource": "cargo test",
                            "summary": "Run cargo test inside the workspace",
                            "content_hash": "sha256:preview",
                            "truncated": false,
                            "redacted": true
                        },
                        "max_duration_seconds": 300,
                        "expires_at": null
                    }
                }
            }
        })
    );
}

#[test]
fn resolve_approval_command_serializes_and_validates() {
    let command = command_envelope(UiCommand::ResolveApproval {
        request_id: id::<ApprovalRequestId>("00000000-0000-0000-0000-000000000001"),
        decision: UiApprovalDecision::Allow,
        reason: "approved by user".to_string(),
        lease_duration_seconds: Some(60),
    });

    validate_ui_command(&command).expect("valid resolve approval command");

    assert_eq!(
        serde_json::to_value(command).expect("serialize command"),
        json!({
            "schema_version": 1,
            "command_id": "00000000-0000-0000-0000-000000000100",
            "client": {
                "kind": "desktop",
                "version": "0.1.0"
            },
            "command": {
                "type": "resolve_approval",
                "data": {
                    "request_id": "00000000-0000-0000-0000-000000000001",
                    "decision": "allow",
                    "reason": "approved by user",
                    "lease_duration_seconds": 60
                }
            }
        })
    );
}

#[test]
fn resolve_approval_command_rejects_empty_reason() {
    let command = command_envelope(UiCommand::ResolveApproval {
        request_id: id::<ApprovalRequestId>("00000000-0000-0000-0000-000000000001"),
        decision: UiApprovalDecision::Deny,
        reason: " ".to_string(),
        lease_duration_seconds: None,
    });

    let error = validate_ui_command(&command).expect_err("empty reason rejected");

    assert!(error.to_string().contains("reason"));
}

#[test]
fn approval_response_command_converts_to_policy_response() {
    let command = UiCommand::ResolveApproval {
        request_id: id::<ApprovalRequestId>("00000000-0000-0000-0000-000000000001"),
        decision: UiApprovalDecision::Deny,
        reason: "not this command".to_string(),
        lease_duration_seconds: Some(60),
    };

    let response = command
        .to_approval_response(timestamp())
        .expect("policy approval response");

    assert_eq!(
        response,
        ApprovalResponse::deny(
            id::<ApprovalRequestId>("00000000-0000-0000-0000-000000000001"),
            "not this command",
            timestamp(),
        )
        .expect("expected denial response")
    );
}
