use agent_desktop::{DesktopTaskItem, build_task_monitor};
use agent_types::{
    ConversationId, PlanTodoItemId, PlanTodoProgress, TaskId, Timestamp, TodoPriority, WorkspaceId,
};
use agent_ui_bridge::{
    UiTaskActiveTodoSummary, UiTaskScheduleSummary, UiTaskSnapshot, UiTaskSummary,
};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-07-02T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000006131"
        .parse()
        .expect("workspace id")
}

fn conversation_id() -> ConversationId {
    "00000000-0000-0000-0000-000000006132"
        .parse()
        .expect("conversation id")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000006133"
        .parse()
        .expect("task id")
}

fn ui_task_with_plan() -> UiTaskSummary {
    serde_json::from_value(serde_json::json!({
        "task_id": task_id(),
        "workspace_id": workspace_id(),
        "conversation_id": conversation_id(),
        "state": "waiting_approval",
        "priority": "high",
        "schedule": UiTaskScheduleSummary {
            kind: "one_shot".to_string(),
            label: "once".to_string(),
        },
        "prompt_summary": "Review plan token sk-secret",
        "latest_run_summary": "waiting for approval",
        "plan_progress": PlanTodoProgress {
            total: 5,
            pending: 2,
            in_progress: 1,
            completed: 2,
            cancelled: 0,
        },
        "active_todo": UiTaskActiveTodoSummary::new(
                PlanTodoItemId::from_string_unchecked("todo-active"),
                "Apply patch with api_key hidden",
                TodoPriority::High,
                3,
            )
            .expect("active todo"),
        "waiting_approval": true,
        "latest_notification": "needs command approval",
        "artifact_count": 4,
        "created_at": ts(1),
        "updated_at": ts(2),
    }))
    .expect("ui task")
}

#[test]
fn desktop_task_monitor_preserves_plan_todo_fields() {
    let snapshot = UiTaskSnapshot::new(vec![ui_task_with_plan()], Vec::new()).expect("snapshot");

    let monitor = build_task_monitor(&snapshot).expect("monitor");
    let task = &monitor.tasks[0];

    assert_eq!(task.plan_progress.expect("progress").total, 5);
    assert_eq!(
        task.active_todo
            .as_ref()
            .expect("active")
            .item_id
            .to_string(),
        "todo-active"
    );
    assert_eq!(
        task.active_todo.as_ref().expect("active").priority,
        TodoPriority::High
    );
    assert!(
        task.active_todo
            .as_ref()
            .expect("active")
            .content
            .contains("[redacted]")
    );
    assert!(
        !task
            .active_todo
            .as_ref()
            .expect("active")
            .content
            .contains("api_key")
    );
    assert!(task.waiting_approval);
    assert_eq!(
        task.latest_notification.as_deref(),
        Some("needs command approval")
    );
    assert_eq!(task.artifact_count, 4);
    assert_eq!(monitor.counts.waiting_approval, 1);
}

#[test]
fn desktop_task_item_try_from_ui_summary_includes_plan_fields() {
    let item = DesktopTaskItem::try_from(&ui_task_with_plan()).expect("desktop task");
    let serialized = serde_json::to_string(&item).expect("serialize task item");

    assert_eq!(item.plan_progress.expect("progress").completed, 2);
    assert_eq!(item.active_todo.as_ref().expect("active").evidence_count, 3);
    assert!(item.waiting_approval);
    assert_eq!(
        item.latest_notification.as_deref(),
        Some("needs command approval")
    );
    assert_eq!(item.artifact_count, 4);
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("api_key"));
}
