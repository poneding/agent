use agent_desktop::{
    DesktopConversationItem, DesktopError, DesktopTaskItem, build_workspace_snapshot,
};
use agent_types::{ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};

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
