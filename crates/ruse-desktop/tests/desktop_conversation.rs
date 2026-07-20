use ruse_desktop::{
    DesktopBridgeErrorNotice, DesktopConversationFrameUpdate, DesktopConversationItem,
    DesktopConversationPanel, DesktopTranscriptEntry, DesktopTranscriptRole, DesktopTurnCommandAck,
    DesktopTurnSubmission, DesktopTurnSubmissionStatus, conversation_update_from_frame,
};
use ruse_types::{
    ConversationId, CorrelationId, EventId, EventSeq, SchemaVersion, Timestamp, WorkspaceId,
};
use ruse_ui_bridge::{UiDiagnosticLevel, UiEventEnvelope, UiEventPayload, UiJsonFrame};

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

fn event_frame(payload: UiEventPayload) -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: id::<EventId>("00000000-0000-0000-0000-000000000004"),
        seq: Some(EventSeq::new(1).expect("valid sequence")),
        occurred_at: Timestamp::now_utc(),
        conversation_id: conversation_id(),
        turn_id: None,
        task_id: None,
        correlation_id: command_id(),
        payload,
    })
}

#[test]
fn prompt_preview_is_bounded_and_redacted() {
    let prompt = format!(
        "inspect token sk-test-secret password api_key {}",
        "x".repeat(400)
    );

    let submission =
        DesktopTurnSubmission::submitting(command_id(), conversation_id(), prompt).unwrap();
    let serialized = serde_json::to_string(&submission).expect("serialize submission");

    assert_eq!(submission.status, DesktopTurnSubmissionStatus::Submitting);
    assert!(submission.truncated);
    assert!(submission.prompt_preview.chars().count() <= 240);
    assert!(submission.prompt_preview.contains("[redacted]"));
    assert!(!serialized.contains("sk-test"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("api_key"));
}

#[test]
fn transcript_entries_serialize_roles_as_snake_case() {
    let entry = DesktopTranscriptEntry::new(
        "event-1",
        Some(conversation_id()),
        DesktopTranscriptRole::Assistant,
        "delta text",
        Some("event-1".to_string()),
    )
    .unwrap();

    let value = serde_json::to_value(entry).expect("serialize transcript entry");

    assert_eq!(value["role"], "assistant");
    assert_eq!(value["content"], "delta text");
}

#[test]
fn frame_updates_cover_transcript_done_and_error_frames() {
    let transcript = conversation_update_from_frame(&event_frame(UiEventPayload::ModelDelta {
        delta: "hello".to_string(),
    }))
    .unwrap();
    assert!(matches!(
        transcript,
        DesktopConversationFrameUpdate::Transcript(DesktopTranscriptEntry {
            role: DesktopTranscriptRole::Assistant,
            ..
        })
    ));

    let done = conversation_update_from_frame(&UiJsonFrame::Done { event_count: 3 }).unwrap();
    assert_eq!(
        done,
        DesktopConversationFrameUpdate::Done { event_count: 3 }
    );

    let error = conversation_update_from_frame(&UiJsonFrame::Error {
        code: "desktop.bridge.test".to_string(),
        message: "token sk-test-secret should not leak".to_string(),
        recoverable: true,
    })
    .unwrap();
    let serialized = serde_json::to_string(&error).expect("serialize error update");

    assert!(matches!(
        error,
        DesktopConversationFrameUpdate::Error(DesktopBridgeErrorNotice {
            recoverable: true,
            ..
        })
    ));
    assert!(!serialized.contains("sk-test"));
    assert!(!serialized.contains("token"));
    assert!(serialized.contains("[redacted]"));
}

#[test]
fn conversation_panel_applies_ack_done_and_recoverable_errors() {
    let conversation =
        DesktopConversationItem::new(conversation_id(), "Main thread", Timestamp::now_utc(), 0)
            .unwrap();
    let mut panel =
        DesktopConversationPanel::new(workspace_id(), Some(conversation_id()), vec![conversation])
            .unwrap();

    panel
        .record_submission(
            DesktopTurnSubmission::submitting(command_id(), conversation_id(), "hello").unwrap(),
        )
        .unwrap();
    panel
        .apply_ack(DesktopTurnCommandAck::new(command_id(), true, 2))
        .unwrap();
    assert_eq!(
        panel.current_submission.as_ref().unwrap().status,
        DesktopTurnSubmissionStatus::Accepted
    );

    panel
        .apply_frame(&event_frame(UiEventPayload::UserMessage {
            message: "hello".to_string(),
        }))
        .unwrap();
    panel
        .apply_frame(&event_frame(UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Warning,
            code: "desktop.test".to_string(),
            message: "working".to_string(),
            recoverable: true,
        }))
        .unwrap();
    panel
        .apply_frame(&UiJsonFrame::Done { event_count: 2 })
        .unwrap();

    assert_eq!(panel.transcript.len(), 2);
    assert_eq!(
        panel.current_submission.as_ref().unwrap().status,
        DesktopTurnSubmissionStatus::Completed
    );

    panel
        .record_submission(
            DesktopTurnSubmission::submitting(command_id(), conversation_id(), "retry").unwrap(),
        )
        .unwrap();
    panel
        .apply_frame(&UiJsonFrame::Error {
            code: "desktop.bridge.disconnected".to_string(),
            message: "desktop bridge session is not connected".to_string(),
            recoverable: true,
        })
        .unwrap();

    assert_eq!(panel.diagnostics.len(), 1);
    assert_eq!(
        panel.current_submission.as_ref().unwrap().status,
        DesktopTurnSubmissionStatus::Rejected
    );
}
