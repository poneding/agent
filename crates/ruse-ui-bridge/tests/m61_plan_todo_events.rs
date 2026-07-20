use ruse_types::{
    ActorRef, ConversationId, CorrelationId, EventId, PlanTodoProgress, PlanTodoSnapshot,
    PlanTodoSnapshotRecorded, PlanTodoUpdated, RuntimeEvent, RuntimeEventPayload, SchemaVersion,
    Timestamp, WorkspaceId,
};
use ruse_ui_bridge::{UiEventPayload, normalize_runtime_event};

fn ts() -> Timestamp {
    serde_json::from_str("\"2026-07-01T00:00:00Z\"").expect("timestamp")
}

fn runtime_event(payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::new(),
        seq: None,
        occurred_at: ts(),
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: None,
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload,
    }
}

#[test]
fn plan_todo_snapshot_event_normalizes_to_progress() {
    let event = runtime_event(RuntimeEventPayload::PlanTodoSnapshotRecorded(
        PlanTodoSnapshotRecorded {
            snapshot: PlanTodoSnapshot {
                conversation_id: ConversationId::new(),
                turn_id: None,
                task_id: None,
                revision: 7,
                items: Vec::new(),
                updated_at: ts(),
                diagnostics: Vec::new(),
            },
            progress: PlanTodoProgress {
                total: 3,
                pending: 1,
                in_progress: 1,
                completed: 1,
                cancelled: 0,
            },
        },
    ));

    let envelope = normalize_runtime_event(&event);

    assert_eq!(
        envelope.payload,
        UiEventPayload::Progress {
            message: "plan/todo snapshot revision=7 items=3 in_progress=1".to_string()
        }
    );
}

#[test]
fn plan_todo_updated_event_normalizes_to_redacted_progress() {
    let event = runtime_event(RuntimeEventPayload::PlanTodoUpdated(PlanTodoUpdated {
        conversation_id: ConversationId::new(),
        turn_id: None,
        task_id: None,
        previous_revision: 7,
        next_revision: 8,
        reason: "started sk-secret-token".to_string(),
        progress: PlanTodoProgress::default(),
    }));

    let envelope = normalize_runtime_event(&event);

    assert_eq!(
        envelope.payload,
        UiEventPayload::Progress {
            message: "plan/todo updated revision 7 -> 8: started [redacted]".to_string()
        }
    );
}
