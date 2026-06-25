use agent_core::{TraceQuery, build_trace};
use agent_storage::{EventStore, InMemoryEventStore};
use agent_types::{
    ActorRef, CheckpointId, CheckpointRecorded, ConversationId, CorrelationId,
    FinalResponseProduced, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp,
    ToolCallProposed, TurnId, WorkspaceId,
};
use serde_json::json;

fn runtime_event(conversation_id: ConversationId, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
        conversation_id,
        turn_id: Some(TurnId::new()),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload,
    }
}

fn all_trace(conversation_id: ConversationId) -> TraceQuery {
    TraceQuery {
        conversation_id,
        include_tools: true,
        include_policy: true,
        include_checkpoints: true,
        include_final: true,
    }
}

#[test]
fn trace_query_returns_empty_trace_for_empty_conversation() {
    let store = InMemoryEventStore::default();

    let entries = build_trace(&store, all_trace(ConversationId::new())).expect("build trace");

    assert!(entries.is_empty());
}

#[test]
fn trace_query_filters_tool_and_policy_events() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    store
        .append(runtime_event(
            conversation_id,
            RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
                call_id: "call-1".to_string(),
                tool_name: "workspace.read_file".to_string(),
                input_json: json!({ "path": "README.md" }),
            }),
        ))
        .expect("append tool event");
    store
        .append(runtime_event(
            conversation_id,
            RuntimeEventPayload::PermissionDecisionRecorded(
                agent_types::PermissionDecisionRecorded {
                    request_id: "request-1".to_string(),
                    decision: "allow".to_string(),
                    reason: "read-only".to_string(),
                },
            ),
        ))
        .expect("append policy event");

    let tool_only = build_trace(
        &store,
        TraceQuery {
            conversation_id,
            include_tools: true,
            include_policy: false,
            include_checkpoints: false,
            include_final: false,
        },
    )
    .expect("tool trace");
    let policy_only = build_trace(
        &store,
        TraceQuery {
            conversation_id,
            include_tools: false,
            include_policy: true,
            include_checkpoints: false,
            include_final: false,
        },
    )
    .expect("policy trace");

    assert_eq!(tool_only.len(), 1);
    assert_eq!(tool_only[0].kind, "tool_call");
    assert_eq!(policy_only.len(), 1);
    assert_eq!(policy_only[0].kind, "permission");
}

#[test]
fn trace_query_renders_checkpoint_and_final_entries() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let checkpoint_id = CheckpointId::new();
    store
        .append(runtime_event(
            conversation_id,
            RuntimeEventPayload::CheckpointRecorded(CheckpointRecorded {
                checkpoint_id,
                label: "before shell".to_string(),
            }),
        ))
        .expect("append checkpoint");
    store
        .append(runtime_event(
            conversation_id,
            RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
                message: "done".to_string(),
            }),
        ))
        .expect("append final");

    let entries = build_trace(&store, all_trace(conversation_id)).expect("build trace");

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].seq, 1);
    assert_eq!(entries[0].kind, "checkpoint");
    assert_eq!(
        entries[0].summary,
        format!("checkpoint {checkpoint_id} recorded: before shell")
    );
    assert_eq!(entries[1].seq, 2);
    assert_eq!(entries[1].kind, "final");
    assert_eq!(entries[1].summary, "final response: done");
}

#[test]
fn trace_query_redacts_secret_like_tool_inputs() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    store
        .append(runtime_event(
            conversation_id,
            RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
                call_id: "call-secret".to_string(),
                tool_name: "workspace.read_file".to_string(),
                input_json: json!({
                    "path": "README.md",
                    "api_key": "secret-value",
                    "nested": { "token": "another-secret" }
                }),
            }),
        ))
        .expect("append tool event");

    let entries = build_trace(&store, all_trace(conversation_id)).expect("build trace");

    assert_eq!(entries.len(), 1);
    assert!(entries[0].summary.contains("[redacted]"));
    assert!(!entries[0].summary.contains("secret-value"));
    assert!(!entries[0].summary.contains("another-secret"));
}
