use agent_types::{
    ActorRef, ConversationId, CorrelationId, EventId, RuntimeEvent, RuntimeEventPayload,
    SchemaVersion, TaskId, TaskNotificationRecorded, Timestamp, WorkspaceId,
};
use agent_ui_bridge::{UiEventPayload, normalize_runtime_event};
use time::macros::datetime;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn runtime_event(task_id: TaskId, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: id::<EventId>("00000000-0000-0000-0000-000000000001"),
        seq: None,
        occurred_at: Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC)),
        workspace_id: id::<WorkspaceId>("00000000-0000-0000-0000-000000000002"),
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000003"),
        turn_id: None,
        task_id: Some(task_id),
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000004"),
        causation_id: None,
        payload,
    }
}

#[test]
fn m08_task_notifications_normalize_to_ui_payload() {
    let task_id = id::<TaskId>("00000000-0000-0000-0000-000000000009");
    let event = runtime_event(
        task_id,
        RuntimeEventPayload::TaskNotificationRecorded(TaskNotificationRecorded {
            task_id,
            kind: "completed".to_string(),
            summary: "workspace indexing completed".to_string(),
        }),
    );

    let ui_event = normalize_runtime_event(&event);

    assert_eq!(ui_event.task_id, Some(task_id));
    assert_eq!(
        ui_event.payload,
        UiEventPayload::TaskNotification {
            task_id,
            kind: "completed".to_string(),
            summary: "workspace indexing completed".to_string(),
        }
    );
}

#[test]
fn m08_task_notifications_ui_payload_excludes_raw_secret_like_content() {
    let task_id = id::<TaskId>("00000000-0000-0000-0000-000000000009");
    let event = runtime_event(
        task_id,
        RuntimeEventPayload::TaskNotificationRecorded(TaskNotificationRecorded {
            task_id,
            kind: "waiting_approval".to_string(),
            summary: "waiting for approval".to_string(),
        }),
    );

    let serialized =
        serde_json::to_string(&normalize_runtime_event(&event)).expect("serialize ui event");

    assert!(!serialized.contains("api_token"));
    assert!(!serialized.contains("secret-token"));
    assert!(serialized.contains("waiting_approval"));
}
