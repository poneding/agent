use agent_scheduler::{
    InMemoryTaskNotificationSink, SchedulerError, TaskNotification, TaskNotificationKind,
    TaskNotificationSink,
};
use agent_types::{TaskId, Timestamp};

fn ts() -> Timestamp {
    serde_json::from_str("\"2026-06-25T00:00:00Z\"").expect("valid fixed timestamp")
}

#[test]
fn notifications_validate_and_serialize_stable_shape() {
    let notification = TaskNotification::new(
        TaskId::new(),
        TaskNotificationKind::WaitingApproval,
        "waiting for tool approval",
        ts(),
    )
    .unwrap();
    let invalid = TaskNotification::new(TaskId::new(), TaskNotificationKind::Completed, " ", ts())
        .unwrap_err();
    let value = serde_json::to_value(&notification).expect("serialize notification");

    assert_eq!(value["kind"], "waiting_approval");
    assert_eq!(value["summary"], "waiting for tool approval");
    assert_eq!(value["occurred_at"], "2026-06-25T00:00:00Z");
    assert!(matches!(invalid, SchedulerError::InvalidTask(_)));
}

#[test]
fn notifications_in_memory_sink_records_in_order() {
    let first =
        TaskNotification::new(TaskId::new(), TaskNotificationKind::Queued, "queued", ts()).unwrap();
    let second = TaskNotification::new(
        first.task_id,
        TaskNotificationKind::Completed,
        "completed",
        ts(),
    )
    .unwrap();
    let mut sink = InMemoryTaskNotificationSink::default();

    sink.notify(first.clone()).unwrap();
    sink.notify(second.clone()).unwrap();

    assert_eq!(sink.notifications(), &[first, second]);
}

#[test]
fn notifications_convert_to_runtime_record_shape() {
    let task_id = TaskId::new();
    let notification = TaskNotification::new(
        task_id,
        TaskNotificationKind::QuotaRejected,
        "quota rejected",
        ts(),
    )
    .unwrap();

    let recorded = notification.recorded();

    assert_eq!(recorded.task_id, task_id);
    assert_eq!(recorded.kind, "quota_rejected");
    assert_eq!(recorded.summary, "quota rejected");
}
