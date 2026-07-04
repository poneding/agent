use agent_desktop::{DesktopConversationItem, DesktopTaskItem};
use agent_types::{ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};

use crate::bridge::{DesktopRuntimeSession, DesktopStartSessionRequest};
use crate::workspace_snapshot::{
    build_empty_workspace_snapshot_for_session, build_workspace_snapshot_response,
};

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn now() -> Timestamp {
    Timestamp::now_utc()
}

#[test]
fn empty_workspace_snapshot_uses_connected_session_ids() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);
    let started = session
        .start(DesktopStartSessionRequest {
            workspace_path: None,
            client_version: "0.1.0".to_string(),
        })
        .expect("session starts");

    let snapshot =
        build_empty_workspace_snapshot_for_session(&mut session).expect("snapshot builds");

    assert_eq!(snapshot.workspace_id.to_string(), started.workspace_id);
    assert_eq!(
        snapshot
            .active_conversation_id
            .expect("active conversation")
            .to_string(),
        started.conversation_id
    );
    assert!(snapshot.conversations.is_empty());
    assert!(snapshot.tasks.is_empty());
}

#[test]
fn workspace_snapshot_sorts_active_tasks_before_terminal_tasks() {
    let workspace_id = id::<WorkspaceId>("00000000-0000-0000-0000-000000000001");
    let conversation_id = id::<ConversationId>("00000000-0000-0000-0000-000000000002");
    let running_task = DesktopTaskItem::new(
        id::<TaskId>("00000000-0000-0000-0000-000000000010"),
        workspace_id,
        Some(conversation_id),
        TaskState::Running,
        "running task",
        now(),
    )
    .expect("valid task");
    let done_task = DesktopTaskItem::new(
        id::<TaskId>("00000000-0000-0000-0000-000000000011"),
        workspace_id,
        Some(conversation_id),
        TaskState::Succeeded,
        "done task",
        now(),
    )
    .expect("valid task");

    let snapshot = build_workspace_snapshot_response(
        workspace_id,
        Some(conversation_id),
        vec![
            DesktopConversationItem::new(conversation_id, "Main", now(), 0)
                .expect("valid conversation"),
        ],
        vec![done_task, running_task],
    )
    .expect("snapshot builds");

    assert_eq!(snapshot.tasks[0].summary, "running task");
    assert_eq!(snapshot.tasks[1].summary, "done task");
}

#[test]
fn invalid_workspace_snapshot_maps_to_ui_safe_error() {
    let error = build_workspace_snapshot_response(
        id::<WorkspaceId>("00000000-0000-0000-0000-000000000001"),
        None,
        vec![DesktopConversationItem {
            conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
            title: " ".to_string(),
            last_activity_at: now(),
            unread_count: 0,
        }],
        Vec::new(),
    )
    .expect_err("invalid title is rejected");

    assert_eq!(error.code, "desktop.workspace_snapshot.invalid");
    assert!(error.message.contains("conversation title cannot be empty"));
}

#[test]
fn workspace_snapshot_serializes_stable_shape() {
    let workspace_id = id::<WorkspaceId>("00000000-0000-0000-0000-000000000001");
    let conversation_id = id::<ConversationId>("00000000-0000-0000-0000-000000000002");
    let snapshot = build_workspace_snapshot_response(
        workspace_id,
        Some(conversation_id),
        Vec::new(),
        Vec::new(),
    )
    .expect("snapshot builds");

    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(value["workspace_id"], workspace_id.to_string());
    assert_eq!(value["active_conversation_id"], conversation_id.to_string());
    assert!(value["conversations"].as_array().expect("array").is_empty());
    assert!(value["tasks"].as_array().expect("array").is_empty());
}
