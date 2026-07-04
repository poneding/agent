use agent_core::{RuntimeIds, TurnEngine, TurnInput};
use agent_model::{FakeModelProvider, FakeModelScript};
use agent_storage::{EventStore, InMemoryEventStore};
use agent_types::{ConversationId, CorrelationId, RuntimeEventPayload, TurnId, WorkspaceId};

fn runtime_ids() -> RuntimeIds {
    RuntimeIds {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
    }
}

#[tokio::test]
async fn spine_integration_persists_fake_turn_events() {
    let ids = runtime_ids();
    let provider = FakeModelProvider::new(FakeModelScript::text("hello from fake model"));
    let input = TurnInput {
        ids,
        user_message: "say hello".to_string(),
        provider: "fake".to_string(),
        model: "fake-scripted".to_string(),
    };

    let outcome = TurnEngine::default()
        .run(input, &provider)
        .await
        .expect("run fake turn");

    let mut store = InMemoryEventStore::default();
    for event in outcome.events {
        store.append(event).expect("persist runtime event");
    }

    let persisted = store
        .list_by_conversation(ids.conversation_id)
        .expect("load persisted events");

    assert!(!persisted.is_empty());
    for (index, stored) in persisted.iter().enumerate() {
        assert_eq!(
            stored.event.seq.expect("stored event sequence").get(),
            index as u64 + 1
        );
    }
    assert!(persisted.iter().any(|stored| matches!(
        &stored.event.payload,
        RuntimeEventPayload::FinalResponseProduced(final_response)
            if final_response.message == "hello from fake model"
    )));
}
