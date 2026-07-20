use ruse_core::{ConversationCompactionInput, compact_conversation_events};
use ruse_storage::{EventStore, InMemoryEventStore};
use ruse_types::{
    ActorRef, ConversationId, CorrelationId, EventId, FinalResponseProduced, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, TurnId, UserInputReceived, WorkspaceId,
};

fn runtime_event(
    conversation_id: ConversationId,
    turn_id: TurnId,
    payload: RuntimeEventPayload,
) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::new(),
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

#[test]
fn compact_conversation_events_builds_redacted_bounded_summary() {
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let mut store = InMemoryEventStore::default();
    store
        .append(runtime_event(
            conversation_id,
            turn_id,
            RuntimeEventPayload::UserInputReceived(UserInputReceived {
                message: "please use token sk-test-secret carefully".to_string(),
            }),
        ))
        .expect("append user event");
    store
        .append(runtime_event(
            conversation_id,
            turn_id,
            RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
                message: "I will keep the password hidden in the summary".to_string(),
            }),
        ))
        .expect("append assistant event");

    let report = compact_conversation_events(
        &store,
        ConversationCompactionInput {
            conversation_id,
            max_summary_tokens: 8,
        },
    )
    .expect("compact conversation");

    assert_eq!(report.conversation_id, conversation_id);
    assert_eq!(report.source_message_count, 2);
    assert_eq!(report.summary_message_count, 1);
    assert_eq!(report.dropped_message_count, 2);
    assert!(report.estimated_tokens <= 8);
    let summary = report.summary.expect("summary");
    assert!(summary.contains("[redacted]"));
    assert!(!summary.contains("sk-test-secret"));
    assert!(!summary.contains("password"));
}

#[test]
fn compact_conversation_events_reports_empty_conversation() {
    let conversation_id = ConversationId::new();
    let store = InMemoryEventStore::default();

    let report = compact_conversation_events(
        &store,
        ConversationCompactionInput {
            conversation_id,
            max_summary_tokens: 16,
        },
    )
    .expect("compact empty conversation");

    assert_eq!(report.source_message_count, 0);
    assert_eq!(report.summary_message_count, 0);
    assert_eq!(report.dropped_message_count, 0);
    assert_eq!(report.summary, None);
}
