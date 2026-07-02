use agent_tui::{ComposerAction, TaskPanelState, TuiAction, TuiApp, render_app_to_string};
use agent_types::{
    ConversationId, CorrelationId, EventId, PlanTodoDiagnostic, PlanTodoItem, PlanTodoItemId,
    PlanTodoProgress, PlanTodoSnapshot, RecoveryConflictKind, RecoveryConflictNextAction,
    RecoveryConflictSide, RecoveryConflictSummary, RecoveryDiagnostic, RecoveryDiagnosticSeverity,
    SchemaVersion, TaskId, TaskState, Timestamp, TodoPriority, TodoSource, TodoStatus,
};
use agent_ui_bridge::{
    UiEventEnvelope, UiEventPayload, UiJsonFrame, UiPlanTodoSnapshot, UiTaskActiveTodoSummary,
    UiTaskNotification, UiTaskReviewSnapshot, UiTaskReviewSummary, UiTaskSnapshot,
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

fn task_notification(task_id: TaskId, kind: &str, summary: &str) -> UiJsonFrame {
    event(UiEventPayload::TaskNotification(UiTaskNotification {
        task_id,
        kind: kind.to_string(),
        summary: summary.to_string(),
        approval_id: None,
    }))
}

fn plan_snapshot() -> UiPlanTodoSnapshot {
    let snapshot = PlanTodoSnapshot {
        conversation_id: ConversationId::new(),
        turn_id: None,
        task_id: Some(TaskId::new()),
        revision: 9,
        items: vec![
            PlanTodoItem {
                item_id: PlanTodoItemId::from_string_unchecked("todo-active"),
                content: "Run cargo test with [redacted]".to_string(),
                status: TodoStatus::InProgress,
                priority: TodoPriority::High,
                source: TodoSource::Model,
                task_id: None,
                evidence: Vec::new(),
                updated_at: Timestamp::now_utc(),
            },
            PlanTodoItem {
                item_id: PlanTodoItemId::from_string_unchecked("todo-done"),
                content: "Inspect task panel".to_string(),
                status: TodoStatus::Completed,
                priority: TodoPriority::Normal,
                source: TodoSource::Runtime,
                task_id: None,
                evidence: Vec::new(),
                updated_at: Timestamp::now_utc(),
            },
        ],
        updated_at: Timestamp::now_utc(),
        diagnostics: vec![PlanTodoDiagnostic {
            code: "plan.note".to_string(),
            message: "plan is fixture-backed".to_string(),
        }],
    };
    UiPlanTodoSnapshot::from_snapshot(snapshot, Vec::new()).expect("plan snapshot")
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
    assert!(rendered.contains("Tasks: empty"));
}

#[test]
fn task_panel_renders_plan_todo_snapshot_progress() {
    let mut panel = TaskPanelState::default();

    panel.reduce_frame(&UiJsonFrame::PlanTodoSnapshot(plan_snapshot()));

    let rendered = panel.render_text();
    assert!(
        rendered
            .contains("Plan: revision=9 total=2 pending=0 in_progress=1 completed=1 cancelled=0")
    );
    assert!(rendered.contains("active todo todo-active priority=high"));
    assert!(rendered.contains("Run cargo test with [redacted]"));
    assert!(rendered.contains("plan diagnostic plan.note plan is fixture-backed"));
}

#[test]
fn task_panel_renders_task_snapshot_status_metadata() {
    let mut panel = TaskPanelState::default();
    let snapshot: UiTaskSnapshot = serde_json::from_value(serde_json::json!({
        "tasks": [{
            "task_id": TaskId::new(),
            "workspace_id": agent_types::WorkspaceId::new(),
            "conversation_id": ConversationId::new(),
            "state": "waiting_approval",
            "priority": "high",
            "schedule": { "kind": "one_shot", "label": "once" },
            "prompt_summary": "review workspace",
            "latest_run_summary": "waiting for approval",
            "created_at": "2026-07-01T00:00:00Z",
            "updated_at": "2026-07-01T00:00:01Z"
        }],
        "errors": []
    }))
    .expect("task snapshot");

    panel.reduce_frame(&UiJsonFrame::TaskSnapshot(snapshot));

    let rendered = panel.render_text();
    assert!(rendered.contains("Tasks: 1"));
    assert!(rendered.contains("state waiting_approval count=1"));
    assert!(rendered.contains("priority=high"));
    assert!(rendered.contains("schedule=once"));
    assert!(rendered.contains("latest=waiting for approval"));
}

#[test]
fn task_panel_renders_task_summary_plan_fields() {
    let mut panel = TaskPanelState::default();
    let snapshot: UiTaskSnapshot = serde_json::from_value(serde_json::json!({
        "tasks": [{
            "task_id": TaskId::new(),
            "workspace_id": agent_types::WorkspaceId::new(),
            "conversation_id": ConversationId::new(),
            "state": "waiting_approval",
            "priority": "high",
            "schedule": { "kind": "one_shot", "label": "once" },
            "prompt_summary": "review workspace",
            "latest_run_summary": "waiting for approval",
            "plan_progress": PlanTodoProgress {
                total: 4,
                pending: 1,
                in_progress: 1,
                completed: 2,
                cancelled: 0,
            },
            "active_todo": UiTaskActiveTodoSummary::new(
                PlanTodoItemId::from_string_unchecked("todo-7"),
                "Apply patch",
                TodoPriority::High,
                1,
            ).expect("active todo"),
            "waiting_approval": true,
            "latest_notification": "needs approval",
            "artifact_count": 3,
            "created_at": "2026-07-01T00:00:00Z",
            "updated_at": "2026-07-01T00:00:01Z"
        }],
        "errors": []
    }))
    .expect("task snapshot");

    panel.reduce_frame(&UiJsonFrame::TaskSnapshot(snapshot));

    let rendered = panel.render_text();
    assert!(rendered.contains("plan=2/1/4"));
    assert!(rendered.contains("active_todo=Apply patch"));
    assert!(rendered.contains("waiting_approval=yes"));
    assert!(rendered.contains("notification=needs approval"));
    assert!(rendered.contains("artifacts=3"));
}

#[test]
fn task_panel_renders_recovery_conflict_summaries() {
    let mut panel = TaskPanelState::default();
    let mut snapshot: UiTaskSnapshot = serde_json::from_value(serde_json::json!({
        "tasks": [{
            "task_id": TaskId::new(),
            "workspace_id": agent_types::WorkspaceId::new(),
            "conversation_id": ConversationId::new(),
            "state": "failed",
            "priority": "high",
            "schedule": { "kind": "one_shot", "label": "once" },
            "prompt_summary": "recover conflicted work",
            "latest_run_summary": "blocked by conflict",
            "created_at": "2026-07-01T00:00:00Z",
            "updated_at": "2026-07-01T00:00:01Z"
        }],
        "errors": []
    }))
    .expect("task snapshot");
    snapshot.tasks[0].recovery_conflicts = vec![RecoveryConflictSummary {
        path: "src/[redacted].rs".to_string(),
        kind: RecoveryConflictKind::BothModified,
        lead_change: RecoveryConflictSide {
            source: "workspace".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("[redacted]".to_string()),
        },
        incoming_change: RecoveryConflictSide {
            source: "recovery".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("safe-hash".to_string()),
        },
        next_action: RecoveryConflictNextAction::InspectDiff,
        diagnostics: vec![RecoveryDiagnostic {
            severity: RecoveryDiagnosticSeverity::Warning,
            code: "recovery.conflict.blocks_resume".to_string(),
            message: "manual review required before resume".to_string(),
            source: Some("fixture".to_string()),
            truncated: false,
        }],
    }];

    panel.reduce_frame(&UiJsonFrame::TaskSnapshot(snapshot));

    let rendered = panel.render_text();
    assert!(rendered.contains("conflicts=1"));
    assert!(rendered.contains("conflict=src/[redacted].rs"));
    assert!(rendered.contains("kind=both_modified"));
    assert!(rendered.contains("next_action=inspect_diff"));
    assert!(!rendered.contains("token-secret"));
    assert!(!rendered.contains("api_key"));
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

#[test]
fn task_panel_renders_review_snapshots_from_bridge_frames() {
    let mut panel = TaskPanelState::default();
    let task_id = TaskId::new();
    let review = UiTaskReviewSummary {
        task_id,
        final_state: TaskState::Succeeded,
        trace_id: Some("trace-123".to_string()),
        checkpoint_id: None,
        artifact_ids: vec!["artifact-report".to_string()],
        artifact_count: 1,
        summary: "completed background review".to_string(),
        artifacts: Vec::new(),
        diagnostics: Vec::new(),
    };
    let snapshot = UiTaskReviewSnapshot::new(vec![review], Vec::new()).expect("review snapshot");

    panel.reduce_frame(&UiJsonFrame::TaskReviewSnapshot(snapshot));

    let rendered = panel.render_text();
    assert!(rendered.contains("review"));
    assert!(rendered.contains("state=succeeded"));
    assert!(rendered.contains("completed background review"));
    assert!(rendered.contains("artifacts=1"));
    assert!(rendered.contains("trace=trace-123"));
}
