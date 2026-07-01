use agent_desktop::{
    DesktopConversationItem, DesktopError, DesktopTaskItem, DesktopTaskReviewItem,
    build_task_monitor, build_workspace_snapshot,
};
use agent_types::{ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};
use agent_ui_bridge::{
    UiTaskCommandError, UiTaskReviewSummary, UiTaskReviewSummaryArtifact, UiTaskScheduleSummary,
    UiTaskSnapshot, UiTaskSummary,
};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-25T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000000001"
        .parse()
        .expect("valid workspace id")
}

fn conversation_id(value: u32) -> ConversationId {
    format!("00000000-0000-0000-0000-{value:012}")
        .parse()
        .expect("valid conversation id")
}

fn task_id(value: u32) -> TaskId {
    format!("00000000-0000-0000-0000-{value:012}")
        .parse()
        .expect("valid task id")
}

fn conversation(value: u32, title: &str, second: u8) -> DesktopConversationItem {
    DesktopConversationItem::new(conversation_id(value), title, ts(second), value)
        .expect("valid conversation item")
}

fn task(value: u32, state: TaskState, summary: &str, second: u8) -> DesktopTaskItem {
    DesktopTaskItem::new(
        task_id(value),
        workspace_id(),
        Some(conversation_id(1)),
        state,
        summary,
        ts(second),
    )
    .expect("valid task item")
}

fn ui_task(value: u32, state: TaskState, summary: &str, second: u8) -> UiTaskSummary {
    serde_json::from_value(serde_json::json!({
        "task_id": task_id(value),
        "workspace_id": workspace_id(),
        "conversation_id": conversation_id(1),
        "state": state,
        "priority": "normal",
        "schedule": UiTaskScheduleSummary {
            kind: "one_shot".to_string(),
            label: "run once".to_string(),
        },
        "prompt_summary": summary,
        "latest_run_summary": format!("latest {summary}"),
        "created_at": ts(second),
        "updated_at": ts(second),
    }))
    .expect("valid ui task summary")
}

#[test]
fn conversation_tasks_serializes_snapshot_shape() {
    let snapshot = build_workspace_snapshot(
        workspace_id(),
        Some(conversation_id(1)),
        vec![conversation(1, "Runtime design", 1)],
        vec![task(1, TaskState::Running, "Run verification", 2)],
    )
    .unwrap();

    let value = serde_json::to_value(snapshot).expect("serialize desktop snapshot");

    assert_eq!(
        value["workspace_id"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(
        value["active_conversation_id"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(value["conversations"][0]["title"], "Runtime design");
    assert_eq!(value["conversations"][0]["unread_count"], 1);
    assert_eq!(value["tasks"][0]["state"], "running");
    assert_eq!(value["tasks"][0]["summary"], "Run verification");
}

#[test]
fn conversation_tasks_sort_conversations_by_recent_activity() {
    let snapshot = build_workspace_snapshot(
        workspace_id(),
        None,
        vec![
            conversation(1, "Older", 1),
            conversation(2, "Newest B", 3),
            conversation(3, "Newest A", 3),
        ],
        Vec::new(),
    )
    .unwrap();

    let titles = snapshot
        .conversations
        .iter()
        .map(|item| item.title.as_str())
        .collect::<Vec<_>>();

    assert_eq!(titles, vec!["Newest A", "Newest B", "Older"]);
}

#[test]
fn conversation_tasks_sort_active_tasks_before_terminal_tasks() {
    let snapshot = build_workspace_snapshot(
        workspace_id(),
        None,
        Vec::new(),
        vec![
            task(1, TaskState::Succeeded, "finished", 3),
            task(2, TaskState::Running, "active older", 1),
            task(3, TaskState::Queued, "active newer", 2),
        ],
    )
    .unwrap();

    let summaries = snapshot
        .tasks
        .iter()
        .map(|item| item.summary.as_str())
        .collect::<Vec<_>>();

    assert_eq!(summaries, vec!["active newer", "active older", "finished"]);
}

#[test]
fn conversation_tasks_reject_empty_titles_and_summaries() {
    let empty_title = DesktopConversationItem::new(conversation_id(1), " ", ts(1), 0);
    let empty_summary = DesktopTaskItem::new(
        task_id(1),
        workspace_id(),
        None,
        TaskState::Queued,
        " ",
        ts(1),
    );

    assert!(matches!(empty_title, Err(DesktopError::InvalidSnapshot(_))));
    assert!(matches!(
        empty_summary,
        Err(DesktopError::InvalidSnapshot(_))
    ));
}

#[test]
fn conversation_tasks_snapshot_excludes_raw_prompt_and_tool_input() {
    let snapshot = build_workspace_snapshot(
        workspace_id(),
        Some(conversation_id(1)),
        vec![conversation(1, "Runtime design", 1)],
        vec![task(1, TaskState::Running, "Run verification", 2)],
    )
    .unwrap();

    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!serialized.contains("prompt"));
    assert!(!serialized.contains("tool_input"));
    assert!(!serialized.contains("input_json"));
}

#[test]
fn conversation_tasks_convert_task_review_items() {
    let ui_review = UiTaskReviewSummary {
        task_id: task_id(24),
        final_state: TaskState::Succeeded,
        trace_id: Some("trace-123".to_string()),
        checkpoint_id: None,
        artifact_ids: vec!["artifact-report".to_string()],
        artifact_count: 1,
        summary: "completed background review".to_string(),
        artifacts: vec![UiTaskReviewSummaryArtifact {
            artifact_id: "artifact-report".to_string(),
            kind: "report".to_string(),
            path: Some("artifacts/report.md".to_string()),
            content_hash: Some("sha256:abc123".to_string()),
            summary: "review report".to_string(),
            missing: false,
            diagnostics: Vec::new(),
        }],
        diagnostics: Vec::new(),
    };

    let item = DesktopTaskReviewItem::try_from(&ui_review).expect("desktop review item");

    assert_eq!(item.task_id, task_id(24));
    assert_eq!(item.final_state, TaskState::Succeeded);
    assert_eq!(item.summary, "completed background review");
    assert_eq!(item.artifact_count, 1);
    assert_eq!(item.trace_id.as_deref(), Some("trace-123"));
    assert_eq!(item.artifacts[0].artifact_id, "artifact-report");
    assert_eq!(
        item.artifacts[0].path.as_deref(),
        Some("artifacts/report.md")
    );
}

#[test]
fn conversation_tasks_build_task_monitor_counts_and_details() {
    let snapshot = UiTaskSnapshot::new(
        vec![
            ui_task(1, TaskState::Succeeded, "finished", 1),
            ui_task(2, TaskState::Running, "running", 2),
            ui_task(3, TaskState::WaitingApproval, "approval", 3),
            ui_task(4, TaskState::Failed, "failed", 4),
        ],
        Vec::new(),
    )
    .expect("task snapshot");

    let monitor = build_task_monitor(&snapshot).expect("task monitor");

    assert_eq!(monitor.counts.total, 4);
    assert_eq!(monitor.counts.active, 1);
    assert_eq!(monitor.counts.waiting_approval, 1);
    assert_eq!(monitor.counts.failed, 1);
    assert_eq!(monitor.counts.terminal, 2);
    assert_eq!(monitor.tasks[0].summary, "approval");
    assert_eq!(
        monitor.tasks[0].latest_run_summary.as_deref(),
        Some("latest approval")
    );
}

#[test]
fn conversation_tasks_task_monitor_errors_are_redacted() {
    let snapshot = UiTaskSnapshot::new(
        vec![ui_task(1, TaskState::Running, "running", 1)],
        vec![
            UiTaskCommandError::recoverable(
                "task.error",
                "token sk-secret password should not leak",
            )
            .expect("task error"),
        ],
    )
    .expect("task snapshot");

    let monitor = build_task_monitor(&snapshot).expect("task monitor");
    let serialized = serde_json::to_string(&monitor).expect("serialize monitor");

    assert_eq!(monitor.errors.len(), 1);
    assert!(monitor.errors[0].recoverable);
    assert!(monitor.errors[0].message.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
}
