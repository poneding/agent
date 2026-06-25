use agent_storage::{EventStore, InMemoryEventStore, ReplayCursor};
use agent_types::{
    ActorRef, ConversationId, CorrelationId, EventId, FinalResponseProduced, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, UserInputReceived, WorkspaceId,
};

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
fn replay_reconstructs_final_response() {
    let conversation_id = ConversationId::new();
    let mut store = InMemoryEventStore::default();

    store
        .append(runtime_event(
            conversation_id,
            RuntimeEventPayload::UserInputReceived(UserInputReceived {
                message: "say hello".to_string(),
            }),
        ))
        .expect("persist user input");
    store
        .append(runtime_event(
            conversation_id,
            RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
                message: "hello from fake model".to_string(),
            }),
        ))
        .expect("persist final response");

    let replayed = ReplayCursor::new(conversation_id)
        .collect(&store)
        .expect("replay conversation");
    let final_response = replayed
        .iter()
        .filter_map(|replay_event| match &replay_event.event.payload {
            RuntimeEventPayload::FinalResponseProduced(final_response) => {
                Some(final_response.message.as_str())
            }
            _ => None,
        })
        .next_back();

    assert_eq!(final_response, Some("hello from fake model"));
}
