use agent_core::RecoveryEngine;
use agent_storage::{
    CheckpointRecord, CheckpointStore, EventStore, IdempotencyKey, IdempotencyStore,
    InMemoryEventStore, ToolEffectRecord,
};
use agent_types::{
    ActorRef, CheckpointId, ConversationId, CorrelationId, EventSeq, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, ToolCallProposed, ToolExecutionCompleted,
    TurnId, UserInputReceived, WorkspaceId,
};
use serde_json::json;

fn runtime_event(
    conversation_id: ConversationId,
    turn_id: TurnId,
    payload: RuntimeEventPayload,
) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
        conversation_id,
        turn_id: Some(turn_id),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload,
    }
}

fn user_event(conversation_id: ConversationId, turn_id: TurnId, message: &str) -> RuntimeEvent {
    runtime_event(
        conversation_id,
        turn_id,
        RuntimeEventPayload::UserInputReceived(UserInputReceived {
            message: message.to_string(),
        }),
    )
}

fn tool_call_event(
    conversation_id: ConversationId,
    turn_id: TurnId,
    call_id: &str,
) -> RuntimeEvent {
    runtime_event(
        conversation_id,
        turn_id,
        RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: call_id.to_string(),
            tool_name: "workspace.edit_file".to_string(),
            input_json: json!({ "path": "README.md" }),
        }),
    )
}

fn tool_completed_event(
    conversation_id: ConversationId,
    turn_id: TurnId,
    call_id: &str,
) -> RuntimeEvent {
    runtime_event(
        conversation_id,
        turn_id,
        RuntimeEventPayload::ToolExecutionCompleted(ToolExecutionCompleted {
            call_id: call_id.to_string(),
            output_json: json!({ "applied": true }),
            elapsed_ms: None,
        }),
    )
}

fn checkpoint(
    checkpoint_id: CheckpointId,
    conversation_id: ConversationId,
    turn_id: TurnId,
    event_seq: u64,
    state_json: serde_json::Value,
) -> CheckpointRecord {
    CheckpointRecord {
        checkpoint_id,
        conversation_id,
        turn_id: Some(turn_id),
        label: format!("checkpoint-{event_seq}"),
        event_seq: EventSeq::new(event_seq).expect("event seq"),
        state_json,
        schema_version: SchemaVersion::current(),
        created_at: Timestamp::now_utc(),
    }
}

#[test]
fn recovery_flow_plans_from_start_without_checkpoint() {
    let store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    let plan = RecoveryEngine
        .plan(&store, conversation_id, Some(turn_id))
        .expect("plan recovery");

    assert_eq!(plan.conversation_id, conversation_id);
    assert_eq!(plan.checkpoint_id, None);
    assert_eq!(plan.replay_from.get(), 1);
    assert!(plan.pending_call_ids.is_empty());
}

#[test]
fn recovery_flow_plans_from_latest_checkpoint() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let older_id = CheckpointId::new();
    let latest_id = CheckpointId::new();

    store
        .put_checkpoint(checkpoint(
            older_id,
            conversation_id,
            turn_id,
            1,
            json!({ "state": "older" }),
        ))
        .expect("older checkpoint");
    store
        .put_checkpoint(checkpoint(
            latest_id,
            conversation_id,
            turn_id,
            3,
            json!({ "state": "latest" }),
        ))
        .expect("latest checkpoint");

    let plan = RecoveryEngine
        .plan(&store, conversation_id, Some(turn_id))
        .expect("plan recovery");

    assert_eq!(plan.checkpoint_id, Some(latest_id));
    assert_eq!(plan.replay_from.get(), 4);
}

#[test]
fn recovery_flow_replays_from_checkpoint_tail() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .append(user_event(conversation_id, turn_id, "first"))
        .expect("append first event");
    store
        .append(user_event(conversation_id, turn_id, "second"))
        .expect("append second event");
    store
        .append(user_event(conversation_id, turn_id, "third"))
        .expect("append third event");
    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            conversation_id,
            turn_id,
            1,
            json!({ "state": "after first" }),
        ))
        .expect("checkpoint");

    let outcome = RecoveryEngine
        .recover(&store, &store, conversation_id, Some(turn_id))
        .expect("recover");

    assert!(outcome.resumed);
    assert_eq!(outcome.plan.replay_from.get(), 2);
    assert_eq!(outcome.replayed_event_count, 2);
}

#[test]
fn recovery_flow_resume_reports_runtime_owned_resume() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .append(user_event(
            conversation_id,
            turn_id,
            "resume me with api_key=secret-value",
        ))
        .expect("append event");

    let report = RecoveryEngine.resume(&store, &store, conversation_id, Some(turn_id));

    assert!(report.resumed);
    assert!(!report.blocked);
    assert_eq!(report.replayed_event_count, 1);
    assert_eq!(report.snapshot.conversation_id, conversation_id);
    let continuation = report.continuation.expect("resume continuation");
    assert!(continuation.can_start_turn);
    assert!(continuation.next_prompt.contains("resume me"));
    assert!(continuation.next_prompt.contains("[redacted]"));
    assert!(!continuation.next_prompt.contains("secret-value"));
    assert_eq!(continuation.replayed_event_count, 1);
}

#[test]
fn recovery_flow_excludes_idempotent_completed_calls_from_pending() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .append(user_event(conversation_id, turn_id, "before checkpoint"))
        .expect("append checkpointed event");
    store
        .append(tool_call_event(conversation_id, turn_id, "call-edit"))
        .expect("append pending call");
    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            conversation_id,
            turn_id,
            1,
            json!({ "state": "before tool" }),
        ))
        .expect("checkpoint");
    store
        .record_effect(ToolEffectRecord {
            key: IdempotencyKey::new("call-edit").expect("key"),
            call_id: "call-edit".to_string(),
            tool_name: "workspace.edit_file".to_string(),
            input_hash: "sha256:abc".to_string(),
            output_json: json!({ "applied": true }),
            completed_at: Timestamp::now_utc(),
        })
        .expect("record completed effect");

    let outcome = RecoveryEngine
        .recover(&store, &store, conversation_id, Some(turn_id))
        .expect("recover");

    assert!(outcome.plan.pending_call_ids.is_empty());
    assert_eq!(outcome.replayed_event_count, 1);
}

#[test]
fn recovery_flow_reports_corrupt_checkpoint_state() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            conversation_id,
            turn_id,
            1,
            json!({ "corrupt": true }),
        ))
        .expect("checkpoint");

    let error = RecoveryEngine
        .plan(&store, conversation_id, Some(turn_id))
        .expect_err("corrupt checkpoint rejected");

    assert!(error.to_string().contains("checkpoint state is corrupt"));
}

#[test]
fn recovery_flow_completed_tail_calls_are_not_pending() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .append(user_event(conversation_id, turn_id, "before checkpoint"))
        .expect("append checkpointed event");
    store
        .append(tool_call_event(conversation_id, turn_id, "call-edit"))
        .expect("append call");
    store
        .append(tool_completed_event(conversation_id, turn_id, "call-edit"))
        .expect("append completed call");
    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            conversation_id,
            turn_id,
            1,
            json!({ "state": "before tool" }),
        ))
        .expect("checkpoint");

    let outcome = RecoveryEngine
        .recover(&store, &store, conversation_id, Some(turn_id))
        .expect("recover");

    assert!(outcome.plan.pending_call_ids.is_empty());
    assert_eq!(outcome.replayed_event_count, 2);
}
