use agent_storage::{EventStore, InMemoryEventStore, ReplayCursor};
use agent_types::{
    ActorRef, ConversationId, CorrelationId, EventId, PermissionDecisionRecorded, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, ToolCallProposed, ToolExecutionCompleted,
    ToolExecutionStarted, ToolObservationRecorded, WorkspaceId,
};
use serde_json::json;

fn runtime_event(conversation_id: ConversationId, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
        conversation_id,
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
fn tool_audit_replay_preserves_event_order() {
    let conversation_id = ConversationId::new();
    let mut store = InMemoryEventStore::default();

    let payloads = vec![
        RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: "call-1".to_string(),
            tool_name: "workspace.read_file".to_string(),
            input_json: json!({ "path": "README.md" }),
        }),
        RuntimeEventPayload::PermissionDecisionRecorded(PermissionDecisionRecorded {
            request_id: "request-1".to_string(),
            decision: "allow".to_string(),
            reason: "read-only path".to_string(),
        }),
        RuntimeEventPayload::ToolExecutionStarted(ToolExecutionStarted {
            call_id: "call-1".to_string(),
            tool_name: "workspace.read_file".to_string(),
        }),
        RuntimeEventPayload::ToolExecutionCompleted(ToolExecutionCompleted {
            call_id: "call-1".to_string(),
            output_json: json!({ "content": "hello" }),
        }),
        RuntimeEventPayload::ToolObservationRecorded(ToolObservationRecorded {
            call_id: "call-1".to_string(),
            observation: "read README.md".to_string(),
        }),
    ];

    for payload in payloads {
        store
            .append(runtime_event(conversation_id, payload))
            .expect("append audit event");
    }

    let replayed = ReplayCursor::new(conversation_id)
        .collect(&store)
        .expect("replay audit events");
    let event_names: Vec<&str> = replayed
        .iter()
        .map(|event| match &event.event.payload {
            RuntimeEventPayload::ToolCallProposed(_) => "proposed",
            RuntimeEventPayload::PermissionDecisionRecorded(_) => "decision",
            RuntimeEventPayload::ToolExecutionStarted(_) => "started",
            RuntimeEventPayload::ToolExecutionCompleted(_) => "completed",
            RuntimeEventPayload::ToolObservationRecorded(_) => "observed",
            _ => "unexpected",
        })
        .collect();

    assert_eq!(
        event_names,
        vec!["proposed", "decision", "started", "completed", "observed"]
    );
}
