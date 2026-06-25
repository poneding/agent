use agent_storage::{EventStore, InMemoryEventStore, ReplayCursor, SqliteEventStore, StorageError};
use agent_types::{
    ActorRef, ConversationId, CorrelationId, EventId, EventSeq, RuntimeEvent, RuntimeEventPayload,
    SchemaVersion, Timestamp, UserInputReceived, WorkspaceId,
};

fn runtime_event(conversation_id: ConversationId, message: &str) -> RuntimeEvent {
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
        payload: RuntimeEventPayload::UserInputReceived(UserInputReceived {
            message: message.to_string(),
        }),
    }
}

fn assert_assigns_ordered_sequences<S: EventStore>(store: &mut S) {
    let conversation_id = ConversationId::new();

    let first = store
        .append(runtime_event(conversation_id, "first"))
        .expect("append first event");
    let second = store
        .append(runtime_event(conversation_id, "second"))
        .expect("append second event");

    assert_eq!(first.event.seq.expect("first sequence").get(), 1);
    assert_eq!(second.event.seq.expect("second sequence").get(), 2);

    let events = store
        .list_by_conversation(conversation_id)
        .expect("list conversation events");

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event.id, first.event.id);
    assert_eq!(events[1].event.id, second.event.id);
}

fn assert_rejects_duplicate_event_id<S: EventStore>(store: &mut S) {
    let conversation_id = ConversationId::new();
    let event = runtime_event(conversation_id, "duplicate");
    let event_id = event.id;

    store.append(event.clone()).expect("append original event");
    let error = store.append(event).expect_err("duplicate event rejected");

    assert!(matches!(error, StorageError::DuplicateEvent(id) if id == event_id));
}

fn assert_replay_returns_ordered_events<S: EventStore>(store: &mut S) {
    let conversation_id = ConversationId::new();

    store
        .append(runtime_event(conversation_id, "first"))
        .expect("append first event");
    store
        .append(runtime_event(conversation_id, "second"))
        .expect("append second event");

    let replayed = ReplayCursor::new(conversation_id)
        .collect(store)
        .expect("collect replay events");

    assert_eq!(replayed.len(), 2);
    assert_eq!(replayed[0].seq.get(), 1);
    assert_eq!(replayed[1].seq.get(), 2);
    assert_eq!(replayed[0].event.schema_version, SchemaVersion::current());
}

fn assert_replay_from_sequence_skips_prior_events<S: EventStore>(store: &mut S) {
    let conversation_id = ConversationId::new();

    store
        .append(runtime_event(conversation_id, "first"))
        .expect("append first event");
    store
        .append(runtime_event(conversation_id, "second"))
        .expect("append second event");

    let replayed =
        ReplayCursor::from_sequence(conversation_id, EventSeq::new(2).expect("valid sequence"))
            .collect(store)
            .expect("collect replay events");

    assert_eq!(replayed.len(), 1);
    assert_eq!(replayed[0].seq.get(), 2);
}

#[test]
fn in_memory_assigns_ordered_sequences() {
    let mut store = InMemoryEventStore::default();
    assert_assigns_ordered_sequences(&mut store);
}

#[test]
fn in_memory_rejects_duplicate_event_id() {
    let mut store = InMemoryEventStore::default();
    assert_rejects_duplicate_event_id(&mut store);
}

#[test]
fn sqlite_assigns_ordered_sequences() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite store");
    assert_assigns_ordered_sequences(&mut store);
}

#[test]
fn sqlite_rejects_duplicate_event_id() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite store");
    assert_rejects_duplicate_event_id(&mut store);
}

#[test]
fn in_memory_replay_returns_ordered_events() {
    let mut store = InMemoryEventStore::default();
    assert_replay_returns_ordered_events(&mut store);
}

#[test]
fn sqlite_replay_returns_ordered_events() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite store");
    assert_replay_returns_ordered_events(&mut store);
}

#[test]
fn in_memory_replay_from_sequence_skips_prior_events() {
    let mut store = InMemoryEventStore::default();
    assert_replay_from_sequence_skips_prior_events(&mut store);
}

#[test]
fn sqlite_replay_from_sequence_skips_prior_events() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite store");
    assert_replay_from_sequence_skips_prior_events(&mut store);
}

#[test]
fn replay_empty_conversation_returns_empty_stream() {
    let store = InMemoryEventStore::default();
    let replayed = ReplayCursor::new(ConversationId::new())
        .collect(&store)
        .expect("collect replay events");

    assert!(replayed.is_empty());
}
