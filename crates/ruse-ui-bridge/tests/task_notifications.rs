use ruse_types::{
    ActorRef, ApprovalRequestId, ConversationId, CorrelationId, EventId, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, TaskId, TaskNotificationRecorded, Timestamp, WorkspaceId,
};
use ruse_ui_bridge::{UiEventPayload, UiJsonFrame, normalize_runtime_event};
use time::macros::datetime;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn ts() -> Timestamp {
    Timestamp::from_utc(datetime!(2026-06-28 00:00:00 UTC))
}

fn task_id() -> TaskId {
    id("00000000-0000-0000-0000-000000002401")
}

fn approval_id() -> ApprovalRequestId {
    id("00000000-0000-0000-0000-000000002403")
}

fn runtime_event(payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: id::<EventId>("00000000-0000-0000-0000-000000002411"),
        seq: None,
        occurred_at: ts(),
        workspace_id: id::<WorkspaceId>("00000000-0000-0000-0000-000000002421"),
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000002431"),
        turn_id: None,
        task_id: Some(task_id()),
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: id::<CorrelationId>("00000000-0000-0000-0000-000000002441"),
        causation_id: None,
        payload,
    }
}

#[test]
fn task_notifications_normalize_approval_linked_notification() {
    let event = runtime_event(RuntimeEventPayload::TaskNotificationRecorded(
        TaskNotificationRecorded {
            task_id: task_id(),
            kind: "waiting_approval".to_string(),
            summary: "waiting for command approval".to_string(),
            approval_id: Some(approval_id()),
        },
    ));

    let ui_event = normalize_runtime_event(&event);

    assert_eq!(
        ui_event.payload,
        UiEventPayload::TaskNotification(ruse_ui_bridge::UiTaskNotification {
            task_id: task_id(),
            kind: "waiting_approval".to_string(),
            summary: "waiting for command approval".to_string(),
            approval_id: Some(approval_id()),
        })
    );
}

#[test]
fn task_notifications_json_redacts_secret_like_summary() {
    let event = runtime_event(RuntimeEventPayload::TaskNotificationRecorded(
        TaskNotificationRecorded {
            task_id: task_id(),
            kind: "waiting_approval".to_string(),
            summary: "approve api_key sk-secret-token".to_string(),
            approval_id: Some(approval_id()),
        },
    ));

    let serialized = serde_json::to_string(&UiJsonFrame::Event(normalize_runtime_event(&event)))
        .expect("serialize");

    assert!(serialized.contains("\"type\":\"task_notification\""));
    assert!(serialized.contains("\"approval_id\":\"00000000-0000-0000-0000-000000002403\""));
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret-token"));
    assert!(!serialized.contains("api_key"));
}
