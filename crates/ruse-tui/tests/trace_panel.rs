use ruse_tui::{
    ComposerAction, TraceFilter, TracePanelState, TuiAction, TuiApp, render_app_to_string,
};
use ruse_types::{
    CheckpointId, ConversationId, CorrelationId, EventId, SchemaVersion, TaskId, Timestamp,
    TurnState,
};
use ruse_ui_bridge::{
    UiDiagnosticLevel, UiEventEnvelope, UiEventPayload, UiJsonFrame, UiObservabilitySnapshot,
    UiTaskNotification,
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
fn trace_panel_records_runtime_events_in_order() {
    let mut panel = TracePanelState::default();

    panel.reduce_frame(&event(UiEventPayload::Progress {
        message: "assembling context".to_string(),
    }));
    panel.reduce_frame(&event(UiEventPayload::ToolCall {
        call_id: "call-1".to_string(),
        tool_name: "shell".to_string(),
    }));
    panel.reduce_frame(&event(UiEventPayload::Checkpoint {
        checkpoint_id: CheckpointId::new(),
        label: "after tool".to_string(),
    }));
    panel.reduce_frame(&event(UiEventPayload::ExternalTrace {
        source_kind: "mcp".to_string(),
        source_id: "server-a".to_string(),
        summary: "listed tools".to_string(),
    }));

    assert_eq!(panel.entries.len(), 4);
    assert_eq!(panel.entries[0].title, "progress");
    assert_eq!(panel.entries[1].title, "tool call");
    assert_eq!(panel.entries[2].title, "checkpoint");
    assert_eq!(panel.entries[3].summary, "server-a: listed tools");
}

#[test]
fn trace_filters_keep_diagnostics_visible() {
    let mut panel = TracePanelState::default();

    panel.reduce_frame(&event(UiEventPayload::Progress {
        message: "loading profile".to_string(),
    }));
    panel.reduce_frame(&event(UiEventPayload::ToolResult {
        call_id: "call-1".to_string(),
        ok: false,
        summary: "timeout".to_string(),
    }));
    panel.reduce_frame(&event(UiEventPayload::Diagnostic {
        level: UiDiagnosticLevel::Warning,
        code: "tool.timeout".to_string(),
        message: "command timed out".to_string(),
        recoverable: true,
    }));

    panel.filter = TraceFilter::Tools;
    let visible: Vec<_> = panel
        .visible_entries()
        .iter()
        .map(|entry| entry.title.as_str())
        .collect();

    assert_eq!(visible, vec!["tool result", "tool.timeout"]);
}

#[test]
fn trace_panel_renders_empty_and_populated_states() {
    let mut app = TuiApp::default();

    app.reduce(TuiAction::Composer(ComposerAction::InsertText(
        "/trace".to_string(),
    )));
    app.reduce(TuiAction::Composer(ComposerAction::Submit));

    let empty = render_app_to_string(&app, 96, 18).expect("render trace panel");
    assert!(empty.contains("Trace"));
    assert!(empty.contains("No trace events"));

    app.reduce(TuiAction::BridgeFrame(Box::new(event(
        UiEventPayload::TurnState {
            state: TurnState::ModelStreaming,
            reason: "model streaming".to_string(),
        },
    ))));
    app.reduce(TuiAction::BridgeFrame(Box::new(event(
        UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Error,
            code: "runtime.error".to_string(),
            message: "provider disconnected".to_string(),
            recoverable: true,
        },
    ))));

    let populated = render_app_to_string(&app, 96, 18).expect("render populated trace panel");
    assert!(populated.contains("turn"));
    assert!(populated.contains("model streaming"));
    assert!(populated.contains("runtime.error"));
}

#[test]
fn trace_panel_includes_task_notifications_as_trace_entries() {
    let mut panel = TracePanelState::default();
    let task_id = TaskId::new();

    panel.reduce_frame(&event(UiEventPayload::TaskNotification(
        UiTaskNotification {
            task_id,
            kind: "running".to_string(),
            summary: "background agent started".to_string(),
            approval_id: None,
        },
    )));

    assert_eq!(panel.entries.len(), 1);
    assert_eq!(panel.entries[0].title, "task running");
    assert!(
        panel.entries[0]
            .summary
            .contains("background agent started")
    );
}

#[test]
fn trace_panel_includes_observability_snapshot() {
    let mut panel = TracePanelState::default();

    panel.reduce_frame(&UiJsonFrame::ObservabilitySnapshot(
        UiObservabilitySnapshot {
            schema_version: 1,
            event_count: 31,
            trace_count: 12,
            audit_count: 31,
            telemetry_count: 2,
            replayable: true,
            diagnostics: vec!["no checkpoints".to_string()],
        },
    ));

    assert_eq!(panel.entries.len(), 1);
    assert_eq!(panel.entries[0].title, "observability");
    assert!(panel.entries[0].summary.contains("events=31"));
    assert!(panel.entries[0].summary.contains("replayable=true"));
}
