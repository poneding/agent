use ruse_cli::{
    CliApprovalChoice, CliApprovalParseError, approval_command_from_input, parse_approval_input,
    render_approval_prompt, render_approval_required_batch, render_json_approval_frame,
};
use ruse_types::{
    ApprovalRequestId, ConversationId, CorrelationId, EventId, SchemaVersion, Timestamp,
};
use ruse_ui_bridge::{
    UiApprovalDecision, UiApprovalPreviewSummary, UiApprovalRequest, UiCommand, UiEventEnvelope,
    UiEventPayload, UiJsonFrame,
};

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn timestamp() -> Timestamp {
    serde_json::from_str("\"2026-06-27T10:00:00Z\"").expect("valid timestamp")
}

fn approval_request() -> UiApprovalRequest {
    UiApprovalRequest {
        request_id: "00000000-0000-0000-0000-000000000001".to_string(),
        status: "pending".to_string(),
        action: "execute".to_string(),
        resource_kind: "command".to_string(),
        resource: "cargo test".to_string(),
        reason: "run verification before finishing".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "command".to_string(),
            resource: "cargo test".to_string(),
            summary: "Run cargo test inside the workspace".to_string(),
            content_hash: Some("sha256:preview".to_string()),
            truncated: true,
            redacted: true,
        },
        max_duration_seconds: Some(300),
        expires_at: None,
    }
}

fn approval_event_frame() -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: id::<EventId>("00000000-0000-0000-0000-000000000005"),
        seq: None,
        occurred_at: timestamp(),
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
        turn_id: None,
        task_id: None,
        correlation_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000004"),
        payload: UiEventPayload::ApprovalRequested(Box::new(approval_request())),
    })
}

#[test]
fn approval_prompt_parses_allow_with_optional_duration() {
    assert_eq!(
        parse_approval_input("allow").expect("allow parses"),
        CliApprovalChoice::Allow {
            lease_duration_seconds: None,
        }
    );
    assert_eq!(
        parse_approval_input("a 60").expect("short allow parses"),
        CliApprovalChoice::Allow {
            lease_duration_seconds: Some(60),
        }
    );
}

#[test]
fn approval_prompt_parses_deny_abort_and_invalid_input() {
    assert_eq!(
        parse_approval_input("deny command is too broad").expect("deny parses"),
        CliApprovalChoice::Deny {
            reason: "command is too broad".to_string(),
        }
    );
    assert_eq!(
        parse_approval_input("abort").expect("abort parses"),
        CliApprovalChoice::Abort
    );
    assert_eq!(
        parse_approval_input("deny").expect_err("deny requires reason"),
        CliApprovalParseError::MissingDenyReason
    );
    assert_eq!(
        parse_approval_input("maybe").expect_err("unknown input rejected"),
        CliApprovalParseError::UnknownChoice("maybe".to_string())
    );
}

#[test]
fn approval_prompt_renders_safe_terminal_summary() {
    let rendered = render_approval_prompt(&approval_request());

    assert!(rendered.contains("Approval required"));
    assert!(rendered.contains("Action: execute"));
    assert!(rendered.contains("Resource: command cargo test"));
    assert!(rendered.contains("Reason: run verification before finishing"));
    assert!(rendered.contains("Preview: Run cargo test inside the workspace"));
    assert!(rendered.contains("Hash: sha256:preview"));
    assert!(rendered.contains("Redacted: yes"));
    assert!(rendered.contains("[a]llow"));
    assert!(rendered.contains("[d]eny <reason>"));
    assert!(!rendered.contains("secret"));
}

#[test]
fn approval_prompt_converts_user_input_to_bridge_command() {
    let command =
        approval_command_from_input(&approval_request(), "allow 120").expect("approval command");

    assert_eq!(
        command,
        Some(UiCommand::ResolveApproval {
            request_id: id::<ApprovalRequestId>("00000000-0000-0000-0000-000000000001"),
            decision: UiApprovalDecision::Allow,
            reason: "approved from CLI".to_string(),
            lease_duration_seconds: Some(120),
        })
    );

    let denied =
        approval_command_from_input(&approval_request(), "deny too risky").expect("deny command");
    assert_eq!(
        denied,
        Some(UiCommand::ResolveApproval {
            request_id: id::<ApprovalRequestId>("00000000-0000-0000-0000-000000000001"),
            decision: UiApprovalDecision::Deny,
            reason: "too risky".to_string(),
            lease_duration_seconds: None,
        })
    );

    assert_eq!(
        approval_command_from_input(&approval_request(), "abort").expect("abort has no command"),
        None
    );
}

#[test]
fn batch_mode_fails_clearly_when_approval_is_required() {
    let output = render_approval_required_batch(&approval_request());

    assert!(output.exit);
    assert_ne!(output.status_code, 0);
    assert!(output.stdout.contains("approval required"));
    assert!(output.stdout.contains("cargo test"));
    assert!(output.stdout.contains("rerun interactively"));
}

#[test]
fn json_mode_emits_approval_frame_without_terminal_prompt() {
    let output = render_json_approval_frame(approval_event_frame()).expect("json frame");

    assert!(!output.exit);
    assert_eq!(output.status_code, 0);
    assert!(!output.stdout.contains("[a]llow"));

    let frame: UiJsonFrame =
        serde_json::from_str(output.stdout.trim()).expect("parse approval frame");
    assert!(matches!(
        frame,
        UiJsonFrame::Event(event)
            if matches!(
                &event.payload,
                UiEventPayload::ApprovalRequested(request)
                    if request.request_id == "00000000-0000-0000-0000-000000000001"
            )
    ));
}
