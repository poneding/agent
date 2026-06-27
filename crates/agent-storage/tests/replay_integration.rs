use agent_storage::{EventStore, InMemoryEventStore, ReplayCursor};
use agent_types::{
    ActorRef, ConversationId, CorrelationId, EventId, FinalResponseProduced, ModelOutputDelta,
    RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp, UserInputReceived, WorkspaceId,
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

#[test]
fn replay_reconstructs_streamed_response_from_deltas() {
    let conversation_id = ConversationId::new();
    let mut store = InMemoryEventStore::default();

    for payload in [
        RuntimeEventPayload::ModelOutputDelta(ModelOutputDelta {
            delta: "hel".to_string(),
        }),
        RuntimeEventPayload::ModelOutputDelta(ModelOutputDelta {
            delta: "lo".to_string(),
        }),
        RuntimeEventPayload::FinalResponseProduced(FinalResponseProduced {
            message: "hello".to_string(),
        }),
    ] {
        store
            .append(runtime_event(conversation_id, payload))
            .expect("persist stream event");
    }

    let replayed = ReplayCursor::new(conversation_id)
        .collect(&store)
        .expect("replay conversation");
    let from_deltas = replayed
        .iter()
        .filter_map(|event| match &event.event.payload {
            RuntimeEventPayload::ModelOutputDelta(delta) => Some(delta.delta.as_str()),
            _ => None,
        })
        .collect::<String>();
    let final_response = replayed
        .iter()
        .filter_map(|event| match &event.event.payload {
            RuntimeEventPayload::FinalResponseProduced(response) => Some(response.message.as_str()),
            _ => None,
        })
        .next_back();

    assert_eq!(from_deltas, "hello");
    assert_eq!(final_response, Some(from_deltas.as_str()));
}
