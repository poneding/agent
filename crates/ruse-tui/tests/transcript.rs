use ruse_tui::{TranscriptItemKind, TranscriptState};
use ruse_types::{ConversationId, CorrelationId, EventId, SchemaVersion, Timestamp};
use ruse_ui_bridge::{
    UiApprovalPreviewSummary, UiApprovalRequest, UiDiagnosticLevel, UiEventEnvelope,
    UiEventPayload, UiJsonFrame, UiObservabilitySnapshot,
};

fn event(payload: UiEventPayload) -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        conversation_id: ConversationId::new(),
        turn_id: None,
        task_id: None,
        correlation_id: CorrelationId::new(),
        payload,
    })
}

#[test]
fn transcript_aggregates_model_deltas_and_final_response_closes_stream() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(event(UiEventPayload::ModelDelta {
        delta: "Hel".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ModelDelta {
        delta: "lo".to_string(),
    }));

    assert_eq!(transcript.items.len(), 1);
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::Model);
    assert_eq!(transcript.items[0].body, "Hello");
    assert!(transcript.items[0].streaming);

    transcript.reduce_frame(event(UiEventPayload::FinalResponse {
        message: "Hello".to_string(),
    }));

    assert_eq!(transcript.items.len(), 1);
    assert_eq!(transcript.items[0].body, "Hello");
    assert!(!transcript.items[0].streaming);
}

#[test]
fn transcript_records_user_tool_approval_and_diagnostic_items_in_order() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(event(UiEventPayload::UserMessage {
        message: "run tests".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ToolCall {
        call_id: "call-1".to_string(),
        tool_name: "shell".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ToolResult {
        call_id: "call-1".to_string(),
        ok: true,
        summary: "completed".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ApprovalRequested(Box::new(
        UiApprovalRequest {
            request_id: "approval-1".to_string(),
            status: "pending".to_string(),
            action: "execute".to_string(),
            resource_kind: "command".to_string(),
            resource: "cargo test".to_string(),
            reason: "run verification".to_string(),
            preview: UiApprovalPreviewSummary {
                kind: "command".to_string(),
                resource: "cargo test".to_string(),
                summary: "Run cargo test".to_string(),
                content_hash: Some("sha256:redacted".to_string()),
                truncated: false,
                redacted: true,
            },
            max_duration_seconds: Some(300),
            expires_at: None,
        },
    ))));
    transcript.reduce_frame(event(UiEventPayload::Diagnostic {
        level: UiDiagnosticLevel::Warning,
        code: "runtime.warning".to_string(),
        message: "network unavailable".to_string(),
        recoverable: true,
    }));

    let kinds: Vec<_> = transcript.items.iter().map(|item| item.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TranscriptItemKind::User,
            TranscriptItemKind::ToolCall,
            TranscriptItemKind::ToolResult,
            TranscriptItemKind::Approval,
            TranscriptItemKind::Diagnostic,
        ]
    );
    assert_eq!(transcript.items[0].body, "run tests");
    assert!(transcript.items[1].body.contains("shell"));
    assert!(transcript.items[2].body.contains("completed"));
    assert!(transcript.items[3].body.contains("cargo test"));
    assert!(transcript.items[3].body.contains("redacted"));
    assert!(transcript.items[4].body.contains("network unavailable"));
}

#[test]
fn transcript_converts_error_frames_to_diagnostics() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(UiJsonFrame::Error {
        code: "bridge.error".to_string(),
        message: "connection dropped".to_string(),
        recoverable: true,
    });

    assert_eq!(transcript.items.len(), 1);
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::Diagnostic);
    assert_eq!(transcript.items[0].title, "bridge.error");
    assert!(transcript.items[0].body.contains("connection dropped"));
}

#[test]
fn transcript_records_observability_snapshot_as_progress() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(UiJsonFrame::ObservabilitySnapshot(
        UiObservabilitySnapshot {
            schema_version: 1,
            event_count: 31,
            trace_count: 12,
            audit_count: 31,
            telemetry_count: 2,
            replayable: true,
            diagnostics: Vec::new(),
        },
    ));

    assert_eq!(transcript.items.len(), 1);
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::Progress);
    assert_eq!(transcript.items[0].title, "observability");
    assert!(transcript.items[0].body.contains("trace=12"));
}

#[test]
fn transcript_respects_in_memory_limit() {
    let mut transcript = TranscriptState::with_limit(2);

    transcript.reduce_frame(event(UiEventPayload::UserMessage {
        message: "one".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::UserMessage {
        message: "two".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::UserMessage {
        message: "three".to_string(),
    }));

    assert_eq!(transcript.items.len(), 2);
    assert_eq!(transcript.items[0].body, "two");
    assert_eq!(transcript.items[1].body, "three");
}
