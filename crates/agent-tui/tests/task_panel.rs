use agent_tui::{ComposerAction, TaskPanelState, TuiAction, TuiApp, render_app_to_string};
use agent_types::{ConversationId, CorrelationId, EventId, SchemaVersion, TaskId, Timestamp};
use agent_ui_bridge::{UiEventEnvelope, UiEventPayload, UiJsonFrame, UiTaskNotification};

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

fn task_notification(task_id: TaskId, kind: &str, summary: &str) -> UiJsonFrame {
    event(UiEventPayload::TaskNotification(UiTaskNotification {
        task_id,
        kind: kind.to_string(),
        summary: summary.to_string(),
        approval_id: None,
    }))
}

#[test]
fn task_panel_accumulates_bounded_notifications() {
    let mut panel = TaskPanelState::with_limit(2);
    let task_id = TaskId::new();

    panel.reduce_frame(&task_notification(task_id, "queued", "waiting"));
    panel.reduce_frame(&task_notification(task_id, "running", "started"));
    panel.reduce_frame(&task_notification(task_id, "succeeded", "finished"));

    assert_eq!(panel.notifications.len(), 2);
    assert_eq!(panel.notifications[0].kind, "running");
    assert_eq!(panel.notifications[1].kind, "succeeded");
    assert_eq!(panel.notifications[1].summary, "finished");
}

#[test]
fn task_panel_ignores_non_task_frames() {
    let mut panel = TaskPanelState::default();

    panel.reduce_frame(&event(UiEventPayload::Progress {
        message: "not a task notification".to_string(),
    }));

    assert!(panel.notifications.is_empty());
}

#[test]
fn tasks_slash_command_opens_empty_task_panel() {
    let mut app = TuiApp::default();

    app.reduce(TuiAction::Composer(ComposerAction::InsertText(
        "/tasks".to_string(),
    )));
    app.reduce(TuiAction::Composer(ComposerAction::Submit));

    let rendered = render_app_to_string(&app, 96, 18).expect("render tasks panel");
    assert!(rendered.contains("Tasks"));
    assert!(rendered.contains("No task notifications"));
}

#[test]
fn task_panel_renders_task_notifications_from_bridge_frames() {
    let mut app = TuiApp::default();
    let task_id = TaskId::new();

    app.reduce(TuiAction::Composer(ComposerAction::InsertText(
        "/tasks".to_string(),
    )));
    app.reduce(TuiAction::Composer(ComposerAction::Submit));
    app.reduce(TuiAction::BridgeFrame(Box::new(task_notification(
        task_id,
        "waiting_approval",
        "needs command approval",
    ))));

    let rendered = render_app_to_string(&app, 96, 18).expect("render populated tasks panel");
    assert!(rendered.contains("Tasks"));
    assert!(rendered.contains("waiting_approval"));
    assert!(rendered.contains("needs command approval"));
}
