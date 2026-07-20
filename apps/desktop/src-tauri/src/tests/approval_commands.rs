use ruse_types::ApprovalRequestId;
use ruse_ui_bridge::{UiApprovalDecision, UiCommand};

use crate::approval_commands::DesktopResolveApprovalRequest;
use crate::bridge::{DesktopRuntimeSession, DesktopStartSessionRequest};

fn request_id() -> ApprovalRequestId {
    "00000000-0000-0000-0000-000000000041"
        .parse()
        .expect("valid approval id")
}

#[test]
fn resolve_approval_request_converts_allow_to_bridge_command() {
    let request = DesktopResolveApprovalRequest {
        request_id: request_id().to_string(),
        decision: UiApprovalDecision::Allow,
        reason: "approved in desktop".to_string(),
        lease_duration_seconds: Some(120),
    };

    let envelope = request
        .to_ui_command_envelope("0.1.0")
        .expect("request converts");

    assert_eq!(envelope.client.version, "0.1.0");
    assert!(matches!(
        envelope.command,
        UiCommand::ResolveApproval {
            request_id: parsed_request_id,
            decision: UiApprovalDecision::Allow,
            lease_duration_seconds: Some(120),
            ..
        } if parsed_request_id == request_id()
    ));
}

#[test]
fn resolve_approval_request_rejects_empty_deny_reason() {
    let request = DesktopResolveApprovalRequest {
        request_id: request_id().to_string(),
        decision: UiApprovalDecision::Deny,
        reason: " ".to_string(),
        lease_duration_seconds: None,
    };

    let error = request
        .to_ui_command_envelope("0.1.0")
        .expect_err("empty reason rejected");

    assert_eq!(error.code, "desktop.approval.invalid");
    assert!(error.message.contains("reason cannot be empty"));
}

#[test]
fn resolve_approval_request_rejects_invalid_request_id() {
    let request = DesktopResolveApprovalRequest {
        request_id: "not-a-uuid".to_string(),
        decision: UiApprovalDecision::Allow,
        reason: "approved".to_string(),
        lease_duration_seconds: None,
    };

    let error = request
        .to_ui_command_envelope("0.1.0")
        .expect_err("invalid id rejected");

    assert_eq!(error.code, "desktop.approval.invalid");
    assert!(error.message.contains("invalid approval request id"));
}

#[test]
fn resolve_approval_submits_through_connected_session() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);
    session
        .start(DesktopStartSessionRequest {
            workspace_path: None,
            client_version: "0.1.0".to_string(),
        })
        .expect("session starts");

    let ack = crate::approval_commands::resolve_approval_in_session(
        &mut session,
        DesktopResolveApprovalRequest {
            request_id: request_id().to_string(),
            decision: UiApprovalDecision::Deny,
            reason: "not this time".to_string(),
            lease_duration_seconds: None,
        },
        "0.1.0",
    )
    .expect("approval command accepted");

    assert!(ack.accepted);
    assert_eq!(ack.pending_frame_count, 2);
}

#[test]
fn resolve_approval_diagnostics_are_redacted() {
    let request = DesktopResolveApprovalRequest {
        request_id: "sk-secret-token".to_string(),
        decision: UiApprovalDecision::Allow,
        reason: "approved".to_string(),
        lease_duration_seconds: None,
    };

    let error = request
        .to_ui_command_envelope("0.1.0")
        .expect_err("invalid id rejected");

    assert!(!error.message.contains("sk-secret-token"));
    assert!(error.message.contains("[redacted]"));
}
