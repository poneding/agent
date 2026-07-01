use agent_types::{
    ActorRef, ConversationId, CorrelationId, EventId, PlanTodoEvidence, PlanTodoEvidenceKind,
    PlanTodoItem, PlanTodoItemId, PlanTodoProgress, PlanTodoSnapshot, PlanTodoSnapshotRecorded,
    RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp, TodoPriority, TodoSource,
    TodoStatus, TurnId, WorkspaceId,
};
use time::macros::datetime;
use uuid::Uuid;

fn ts(second: u8) -> Timestamp {
    Timestamp::from_utc(datetime!(2026-07-01 00:00:00 UTC) + time::Duration::seconds(second.into()))
}

fn fixed_event(payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::from_uuid(Uuid::from_u128(1)),
        seq: None,
        occurred_at: ts(0),
        workspace_id: WorkspaceId::from_uuid(Uuid::from_u128(2)),
        conversation_id: ConversationId::from_uuid(Uuid::from_u128(3)),
        turn_id: Some(TurnId::from_uuid(Uuid::from_u128(4))),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::from_uuid(Uuid::from_u128(5)),
        causation_id: None,
        payload,
    }
}

#[test]
fn m61_plan_todo_contracts_serialize_stable_shape() {
    let snapshot = PlanTodoSnapshot {
        conversation_id: ConversationId::from_uuid(Uuid::from_u128(3)),
        turn_id: Some(TurnId::from_uuid(Uuid::from_u128(4))),
        task_id: None,
        revision: 7,
        items: vec![PlanTodoItem {
            item_id: PlanTodoItemId::from_string_unchecked("todo-1"),
            content: "Inspect task state".to_string(),
            status: TodoStatus::InProgress,
            priority: TodoPriority::High,
            source: TodoSource::Model,
            task_id: None,
            evidence: vec![PlanTodoEvidence {
                kind: PlanTodoEvidenceKind::Trace,
                reference: "trace-123".to_string(),
                summary: "trace evidence".to_string(),
            }],
            updated_at: ts(1),
        }],
        updated_at: ts(2),
        diagnostics: Vec::new(),
    };
    let event = fixed_event(RuntimeEventPayload::PlanTodoSnapshotRecorded(
        PlanTodoSnapshotRecorded {
            snapshot,
            progress: PlanTodoProgress {
                total: 1,
                pending: 0,
                in_progress: 1,
                completed: 0,
                cancelled: 0,
            },
        },
    ));

    let value = serde_json::to_value(event).expect("event json");

    assert_eq!(value["payload"]["type"], "plan_todo_snapshot_recorded");
    assert_eq!(value["payload"]["data"]["snapshot"]["revision"], 7);
    assert_eq!(
        value["payload"]["data"]["snapshot"]["items"][0]["status"],
        "in_progress"
    );
    assert_eq!(
        value["payload"]["data"]["snapshot"]["items"][0]["priority"],
        "high"
    );
    assert_eq!(
        value["payload"]["data"]["snapshot"]["items"][0]["source"],
        "model"
    );
    assert_eq!(
        value["payload"]["data"]["snapshot"]["items"][0]["evidence"][0]["kind"],
        "trace"
    );
    assert_eq!(value["payload"]["data"]["progress"]["in_progress"], 1);
}
