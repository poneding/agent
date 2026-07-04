use agent_types::{
    ApprovalRequestId, ClientKind, ConversationId, CorrelationId, EventId, EventSeq, SchemaVersion,
    TaskId, Timestamp, WorkspaceId,
};
use agent_ui_bridge::{
    UiApprovalDecision, UiApprovalPreviewSummary, UiApprovalRequest, UiClientInfo, UiCommand,
    UiCommandEnvelope, UiEventEnvelope, UiEventPayload, UiJsonFrame, UiTaskNotification,
    encode_json_frame, validate_ui_command,
};
use serde_json::json;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn workspace_id() -> WorkspaceId {
    id("00000000-0000-0000-0000-000000000001")
}

fn conversation_id() -> ConversationId {
    id("00000000-0000-0000-0000-000000000002")
}

fn command_id() -> CorrelationId {
    id("00000000-0000-0000-0000-000000000003")
}

fn approval_id() -> ApprovalRequestId {
    id("00000000-0000-0000-0000-000000000004")
}

fn task_id() -> TaskId {
    id("00000000-0000-0000-0000-000000000005")
}

fn event_id(seq: u64) -> EventId {
    id(&format!("00000000-0000-0000-0000-{:012}", seq + 10))
}

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-30T12:00:{second:02}Z\"")).expect("valid timestamp")
}

fn start_turn_command(prompt: &str) -> UiCommandEnvelope {
    UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: command_id(),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::StartTurn {
            workspace_id: workspace_id(),
            conversation_id: conversation_id(),
            prompt: prompt.to_string(),
        },
    }
}

fn resolve_approval_command(reason: &str) -> UiCommandEnvelope {
    UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: command_id(),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::ResolveApproval {
            request_id: approval_id(),
            decision: UiApprovalDecision::Allow,
            reason: reason.to_string(),
            lease_duration_seconds: Some(120),
        },
    }
}

fn event_frame(seq: u64, payload: UiEventPayload) -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: event_id(seq),
        seq: Some(EventSeq::new(seq).expect("valid sequence")),
        occurred_at: ts(u8::try_from(seq).expect("fixture timestamp")),
        conversation_id: conversation_id(),
        turn_id: None,
        task_id: None,
        correlation_id: command_id(),
        payload,
    })
}

#[test]
fn desktop_start_turn_command_serializes_and_validates() {
    let envelope = start_turn_command("Summarize this workspace");

    validate_ui_command(&envelope).expect("valid start turn command");

    let value = serde_json::to_value(envelope).expect("serialize command");
    assert_eq!(
        value,
        json!({
            "schema_version": 1,
            "command_id": "00000000-0000-0000-0000-000000000003",
            "client": {
                "kind": "desktop",
                "version": "0.1.0"
            },
            "command": {
                "type": "start_turn",
                "data": {
                    "workspace_id": "00000000-0000-0000-0000-000000000001",
                    "conversation_id": "00000000-0000-0000-0000-000000000002",
                    "prompt": "Summarize this workspace"
                }
            }
        })
    );
}

#[test]
fn desktop_resolve_approval_command_serializes_and_validates() {
    let envelope = resolve_approval_command("approved in Desktop");

    validate_ui_command(&envelope).expect("valid resolve approval command");

    let value = serde_json::to_value(envelope).expect("serialize command");
    assert_eq!(
        value,
        json!({
            "schema_version": 1,
            "command_id": "00000000-0000-0000-0000-000000000003",
            "client": {
                "kind": "desktop",
                "version": "0.1.0"
            },
            "command": {
                "type": "resolve_approval",
                "data": {
                    "request_id": "00000000-0000-0000-0000-000000000004",
                    "decision": "allow",
                    "reason": "approved in Desktop",
                    "lease_duration_seconds": 120
                }
            }
        })
    );
}

#[test]
fn desktop_resolve_approval_rejects_empty_reason() {
    let error =
        validate_ui_command(&resolve_approval_command("   ")).expect_err("empty reason rejected");

    assert!(
        error
            .to_string()
            .contains("resolve_approval reason cannot be empty")
    );
}

#[test]
fn desktop_start_turn_rejects_empty_prompt() {
    let error = validate_ui_command(&start_turn_command("   ")).expect_err("empty prompt rejected");

    assert!(
        error
            .to_string()
            .contains("start_turn prompt cannot be empty")
    );
}

#[test]
fn transcript_relevant_frames_encode_as_single_line_json() {
    let frames = [
        event_frame(
            1,
            UiEventPayload::UserMessage {
                message: "hello".to_string(),
            },
        ),
        event_frame(
            2,
            UiEventPayload::ModelDelta {
                delta: "hi".to_string(),
            },
        ),
        UiJsonFrame::Done { event_count: 2 },
        UiJsonFrame::Error {
            code: "desktop.bridge.disconnected".to_string(),
            message: "desktop bridge session is not connected".to_string(),
            recoverable: true,
        },
    ];

    for frame in frames {
        let line = encode_json_frame(&frame).expect("encode frame");
        assert!(line.ends_with('\n'));
        assert_eq!(line.matches('\n').count(), 1);
        serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("frame parses");
    }
}

#[test]
fn desktop_m56_shaped_frame_sequence_is_parseable_and_redacted() {
    let approval = UiApprovalRequest {
        request_id: approval_id().to_string(),
        status: "pending".to_string(),
        action: "write".to_string(),
        resource_kind: "file".to_string(),
        resource: "apps/desktop/src/__tests__/desktop-smoke.test.tsx".to_string(),
        reason: "policy requires Desktop review".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "file_edit".to_string(),
            resource: "apps/desktop/src/__tests__/desktop-smoke.test.tsx".to_string(),
            summary: "+assert Desktop golden path evidence\n-[redacted]".to_string(),
            content_hash: Some("sha256:desktop-smoke".to_string()),
            truncated: false,
            redacted: true,
        },
        max_duration_seconds: Some(120),
        expires_at: Some(ts(50)),
    };
    let frames = [
        event_frame(
            1,
            UiEventPayload::UserMessage {
                message: "Update the Desktop smoke fixture and run tests".to_string(),
            },
        ),
        event_frame(
            2,
            UiEventPayload::ModelDelta {
                delta: "Preparing a bounded diff preview.".to_string(),
            },
        ),
        event_frame(
            3,
            UiEventPayload::TaskNotification(UiTaskNotification {
                task_id: task_id(),
                kind: "waiting_approval".to_string(),
                summary: "Review Desktop smoke fixture edit".to_string(),
                approval_id: Some(approval_id()),
            }),
        ),
        event_frame(4, UiEventPayload::ApprovalRequested(Box::new(approval))),
        event_frame(
            5,
            UiEventPayload::ApprovalResolved {
                request_id: approval_id().to_string(),
                status: "resolved".to_string(),
                decision: Some("allow".to_string()),
                reason: Some("approved in Desktop".to_string()),
            },
        ),
        event_frame(
            6,
            UiEventPayload::FinalResponse {
                message: "Desktop smoke fixture completed with tests passing.".to_string(),
            },
        ),
        UiJsonFrame::Done { event_count: 6 },
    ];

    let mut encoded = String::new();
    for frame in frames {
        let line = encode_json_frame(&frame).expect("encode frame");
        assert!(line.ends_with('\n'));
        assert_eq!(line.matches('\n').count(), 1);
        serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("frame parses");
        encoded.push_str(&line);
    }

    assert!(encoded.contains("approval_requested"));
    assert!(encoded.contains("approval_resolved"));
    assert!(encoded.contains("task_notification"));
    assert!(encoded.contains("Desktop smoke fixture completed"));
    for forbidden in ["sk-secret", "api_key", "token", "password"] {
        assert!(
            !encoded.contains(forbidden),
            "encoded frame sequence leaked {forbidden}: {encoded}"
        );
    }
}
