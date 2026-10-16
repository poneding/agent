use ruse_tui::{TranscriptItem, TranscriptItemKind, TranscriptState};
use ruse_types::{ConversationId, CorrelationId, EventId, SchemaVersion, Timestamp};
use ruse_ui_bridge::{
    UiApprovalPreviewSummary, UiApprovalRequest, UiDiagnosticLevel, UiEventEnvelope,
    UiEventPayload, UiJsonFrame,
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

fn approval_requested(request_id: &str, resource: &str) -> UiEventPayload {
    UiEventPayload::ApprovalRequested(Box::new(UiApprovalRequest {
        request_id: request_id.to_string(),
        status: "pending".to_string(),
        action: "execute".to_string(),
        resource_kind: "command".to_string(),
        resource: resource.to_string(),
        reason: "confirmation required".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "command".to_string(),
            resource: resource.to_string(),
            summary: format!("Run {resource}"),
            content_hash: None,
            truncated: false,
            redacted: false,
        },
        max_duration_seconds: None,
        expires_at: None,
    }))
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
        preview: Some("cargo test".to_string()),
    }));
    transcript.reduce_frame(event(UiEventPayload::ToolResult {
        call_id: "call-1".to_string(),
        ok: true,
        summary: "completed".to_string(),
        elapsed_ms: Some(12),
    }));
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
            TranscriptItemKind::ToolResult,
            TranscriptItemKind::Diagnostic,
        ]
    );
    assert_eq!(transcript.items[0].body, "run tests");
    assert!(
        transcript.items[1].body.contains("cargo test")
            || transcript.items[1].body.contains("shell")
    );
    assert!(transcript.items[1].body.contains("completed"));
    assert!(transcript.items[2].body.contains("network unavailable"));
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
fn transcript_ignores_non_chat_frames() {
    let mut transcript = TranscriptState::default();

    // Admin / snapshot frames are intentionally kept out of the chat stream.
    let frame = UiJsonFrame::observability_snapshot(ruse_ui_bridge::UiObservabilitySnapshot {
        schema_version: 1,
        event_count: 31,
        trace_count: 12,
        audit_count: 31,
        telemetry_count: 2,
        replayable: true,
        diagnostics: Vec::new(),
    })
    .expect("observability snapshot frame");
    transcript.reduce_frame(frame);

    assert!(transcript.items.is_empty());
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

#[test]
fn reasoning_resumes_on_a_new_line_after_each_approval_pause() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: "prepare the file".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ToolCall {
        call_id: "write-1".to_string(),
        tool_name: "workspace.write_file".to_string(),
        preview: Some("write file helloworld.py".to_string()),
    }));
    transcript.reduce_frame(event(approval_requested(
        "approval-1",
        "write helloworld.py",
    )));
    transcript.reduce_frame(event(UiEventPayload::ToolResult {
        call_id: "write-1".to_string(),
        ok: true,
        summary: "created helloworld.py".to_string(),
        elapsed_ms: Some(12),
    }));
    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: " Now run it".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ToolCall {
        call_id: "run-1".to_string(),
        tool_name: "workspace.shell_command".to_string(),
        preview: Some("python helloworld.py".to_string()),
    }));
    transcript.reduce_frame(event(approval_requested(
        "approval-2",
        "python helloworld.py",
    )));
    transcript.reduce_frame(event(UiEventPayload::ToolResult {
        call_id: "run-1".to_string(),
        ok: true,
        summary: "Hello, World!".to_string(),
        elapsed_ms: Some(8),
    }));
    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: " The output is correct".to_string(),
    }));

    let reasoning: Vec<_> = transcript
        .items
        .iter()
        .filter(|item| item.kind == TranscriptItemKind::Reasoning)
        .collect();
    assert_eq!(reasoning.len(), 1, "approval pauses keep one Thought item");
    assert_eq!(
        reasoning[0].body,
        "prepare the file  \nNow run it  \nThe output is correct"
    );
}

#[test]
fn reasoning_approval_break_does_not_leak_into_the_next_turn() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: "first turn reasoning".to_string(),
    }));
    transcript.reduce_frame(event(approval_requested("approval-1", "cargo test")));
    transcript.reduce_frame(event(UiEventPayload::ModelDelta {
        delta: "first answer".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::UserMessage {
        message: "next turn".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: "second turn reasoning".to_string(),
    }));

    let latest = transcript
        .items
        .iter()
        .rev()
        .find(|item| item.kind == TranscriptItemKind::Reasoning)
        .expect("second reasoning item");
    assert_eq!(latest.body, "second turn reasoning");
}

#[test]
fn reasoning_stays_open_across_tools_without_flash_new_block() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: "plan A".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ToolCall {
        call_id: "t1".to_string(),
        tool_name: "workspace.read_file".to_string(),
        preview: Some("read file src/lib.rs".to_string()),
    }));
    // Still one streaming reasoning row (not frozen to Thought).
    assert_eq!(
        transcript
            .items
            .iter()
            .filter(|item| item.kind == TranscriptItemKind::Reasoning)
            .count(),
        1
    );
    assert!(
        transcript
            .items
            .iter()
            .any(|item| item.kind == TranscriptItemKind::Reasoning && item.streaming)
    );

    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: " then B".to_string(),
    }));
    // Same block continues — no second Thinking row.
    let reasoning: Vec<_> = transcript
        .items
        .iter()
        .filter(|item| item.kind == TranscriptItemKind::Reasoning)
        .collect();
    assert_eq!(reasoning.len(), 1);
    assert!(reasoning[0].streaming);
    assert_eq!(reasoning[0].body, "plan A then B");

    // Model answer freezes reasoning into a single Thought.
    transcript.reduce_frame(event(UiEventPayload::ModelDelta {
        delta: "answer".to_string(),
    }));
    assert!(!reasoning_still_streaming(&transcript));
    assert_eq!(
        transcript
            .items
            .iter()
            .filter(|item| item.kind == TranscriptItemKind::Reasoning)
            .count(),
        1
    );
}

fn reasoning_still_streaming(transcript: &TranscriptState) -> bool {
    transcript
        .items
        .iter()
        .any(|item| item.kind == TranscriptItemKind::Reasoning && item.streaming)
}

#[test]
fn empty_reasoning_placeholder_is_removed_instead_of_settling_as_thought() {
    let mut transcript = TranscriptState::default();
    // Turn start inserts a streaming placeholder (body == "thinking").
    transcript.push(TranscriptItem::new(
        TranscriptItemKind::Reasoning,
        "thinking",
        "thinking",
        true,
    ));

    // Empty provider deltas must not convert the placeholder into a ghost body.
    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: String::new(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: "   ".to_string(),
    }));
    assert_eq!(transcript.items.len(), 1);
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::Reasoning);
    assert_eq!(transcript.items[0].body, "thinking");
    assert!(transcript.items[0].streaming);

    // When the model answer starts, the unused placeholder disappears entirely
    // (no "Thought [0s · 0 tokens]" row).
    transcript.reduce_frame(event(UiEventPayload::ModelDelta {
        delta: "hello".to_string(),
    }));
    assert_eq!(transcript.items.len(), 1);
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::Model);
    assert_eq!(transcript.items[0].body, "hello");
}

#[test]
fn shell_tool_phases_waiting_running_ran_and_stdout_only() {
    let mut transcript = TranscriptState::default();

    // Proposed (awaiting approval) → Waiting, not Ran.
    transcript.reduce_frame(event(UiEventPayload::ToolCall {
        call_id: "shell-1".to_string(),
        tool_name: "workspace.shell_command".to_string(),
        preview: Some("run ls -la".to_string()),
    }));
    assert_eq!(transcript.items.len(), 1);
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::ToolCall);
    assert!(!transcript.items[0].streaming);
    assert!(transcript.items[0].body.starts_with("shell ls -la"));

    // Started after permission → Running.
    transcript.reduce_frame(event(UiEventPayload::ToolStarted {
        call_id: "shell-1".to_string(),
        tool_name: "workspace.shell_command".to_string(),
        policy: None,
        sandbox: None,
    }));
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::ToolCall);
    assert!(transcript.items[0].streaming);

    // Completed → Ran + multiline stdout only. This is the bridge summary
    // shape produced from the structured shell result.
    transcript.reduce_frame(event(UiEventPayload::ToolResult {
        call_id: "shell-1".to_string(),
        ok: true,
        summary: "exit=0 stdout=total 48\ndrwxr-xr-x  5 dp  staff  160 .".to_string(),
        elapsed_ms: Some(12),
    }));
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::ToolResult);
    assert!(!transcript.items[0].streaming);
    assert!(
        transcript.items[0]
            .body
            .contains("total 48\ndrwxr-xr-x  5 dp  staff  160 ."),
        "shell rows must remain separated: {:?}",
        transcript.items[0].body
    );
    assert!(!transcript.items[0].body.contains("sandbox_profile"));
    assert!(transcript.items[0].tool_output_expanded);

    // Observation after complete must NOT reopen as Running or reintroduce JSON.
    transcript.reduce_frame(event(UiEventPayload::ToolObservation {
        call_id: "shell-1".to_string(),
        observation: r#"{"sandbox_profile":"workspace_seatbelt","status":0,"stderr":"","stdout":"total 48\ndrwxr-xr-x  5 dp  staff  160 ."}"#.to_string(),
    }));
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::ToolResult);
    assert!(!transcript.items[0].streaming);
    assert!(!transcript.items[0].body.contains("sandbox_profile"));
}

#[test]
fn edit_file_result_preserves_diff_and_auto_expands() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(event(UiEventPayload::ToolCall {
        call_id: "edit-1".to_string(),
        tool_name: "workspace.edit_file".to_string(),
        preview: Some("edit file src/main.rs".to_string()),
    }));
    transcript.reduce_frame(event(UiEventPayload::ToolStarted {
        call_id: "edit-1".to_string(),
        tool_name: "workspace.edit_file".to_string(),
        policy: None,
        sandbox: None,
    }));
    transcript.reduce_frame(event(UiEventPayload::ToolResult {
        call_id: "edit-1".to_string(),
        ok: true,
        summary: r#"{"path":"src/main.rs","applied":true,"diff":"--- src/main.rs\n+++ src/main.rs\n@@\n-old\n+new"}"#.to_string(),
        elapsed_ms: Some(5),
    }));

    assert_eq!(transcript.items.len(), 1);
    let item = &transcript.items[0];
    assert_eq!(item.kind, TranscriptItemKind::ToolResult);
    assert!(item.body.starts_with("edit_file src/main.rs"));
    assert!(item.body.contains("--- src/main.rs"));
    assert!(item.body.contains("+new"));
    assert!(
        item.tool_output_expanded,
        "edit diffs must auto-expand in the transcript"
    );
}

#[test]
fn real_reasoning_still_settles_as_thought_with_stats() {
    let mut transcript = TranscriptState::default();

    transcript.reduce_frame(event(UiEventPayload::ReasoningSummaryDelta {
        delta: "consider tradeoffs".to_string(),
    }));
    transcript.reduce_frame(event(UiEventPayload::ModelDelta {
        delta: "answer".to_string(),
    }));

    assert_eq!(transcript.items.len(), 2);
    assert_eq!(transcript.items[0].kind, TranscriptItemKind::Reasoning);
    assert!(!transcript.items[0].streaming);
    assert_eq!(transcript.items[0].body, "consider tradeoffs");
    assert!(transcript.items[0].thinking_tokens > 0);
    assert_eq!(transcript.items[1].kind, TranscriptItemKind::Model);
    assert_eq!(transcript.items[1].body, "answer");
}
