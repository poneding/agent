use ruse_types::{
    ActorRef, ConversationId, CorrelationId, EventId, RuntimeEvent, RuntimeEventPayload,
    SchemaVersion, TaskId, TaskNotificationRecorded, Timestamp, WorkspaceId,
};
use time::macros::datetime;
use uuid::Uuid;

fn fixed_event(task_id: TaskId, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::from_uuid(Uuid::from_u128(1)),
        seq: None,
        occurred_at: Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC)),
        workspace_id: WorkspaceId::from_uuid(Uuid::from_u128(2)),
        conversation_id: ConversationId::from_uuid(Uuid::from_u128(3)),
        turn_id: None,
        task_id: Some(task_id),
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::from_uuid(Uuid::from_u128(4)),
        causation_id: None,
        payload,
    }
}

#[test]
fn m08_task_notifications_serialize_runtime_event_shape() {
    let task_id = TaskId::from_uuid(Uuid::from_u128(9));
    let event = fixed_event(
        task_id,
        RuntimeEventPayload::TaskNotificationRecorded(TaskNotificationRecorded {
            task_id,
            kind: "waiting_approval".to_string(),
            summary: "waiting for tool approval".to_string(),
            approval_id: None,
        }),
    );

    let value = serde_json::to_value(event).expect("serialize task notification event");

    assert_eq!(value["task_id"], "00000000-0000-0000-0000-000000000009");
    assert_eq!(value["payload"]["type"], "task_notification_recorded");
    assert_eq!(
        value["payload"]["data"]["task_id"],
        "00000000-0000-0000-0000-000000000009"
    );
    assert_eq!(value["payload"]["data"]["kind"], "waiting_approval");
    assert_eq!(
        value["payload"]["data"]["summary"],
        "waiting for tool approval"
    );
}
