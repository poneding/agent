use agent_storage::{
    CheckpointRecord, CheckpointStore, InMemoryEventStore, SqliteEventStore, StorageError,
};
use agent_types::{CheckpointId, ConversationId, EventSeq, SchemaVersion, Timestamp, TurnId};
use serde_json::json;

fn checkpoint(
    checkpoint_id: CheckpointId,
    conversation_id: ConversationId,
    turn_id: Option<TurnId>,
    event_seq: u64,
    label: &str,
) -> CheckpointRecord {
    CheckpointRecord {
        checkpoint_id,
        conversation_id,
        turn_id,
        label: label.to_string(),
        event_seq: EventSeq::new(event_seq).expect("event seq"),
        state_json: json!({
            "turn_state": "observed",
            "note": label
        }),
        schema_version: SchemaVersion::current(),
        created_at: Timestamp::now_utc(),
    }
}

fn assert_puts_and_loads_latest_checkpoint<S: CheckpointStore>(store: &mut S) {
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let first_id = CheckpointId::new();
    let latest_id = CheckpointId::new();

    store
        .put_checkpoint(checkpoint(
            first_id,
            conversation_id,
            Some(turn_id),
            1,
            "first",
        ))
        .expect("put first checkpoint");
    store
        .put_checkpoint(checkpoint(
            latest_id,
            conversation_id,
            Some(turn_id),
            3,
            "latest",
        ))
        .expect("put latest checkpoint");

    let latest = store
        .latest_checkpoint(conversation_id, Some(turn_id))
        .expect("load latest")
        .expect("checkpoint exists");

    assert_eq!(latest.checkpoint_id, latest_id);
    assert_eq!(latest.event_seq.get(), 3);
    assert_eq!(latest.label, "latest");
}

fn assert_filters_by_conversation_and_turn<S: CheckpointStore>(store: &mut S) {
    let conversation_id = ConversationId::new();
    let other_conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let other_turn_id = TurnId::new();
    let expected_id = CheckpointId::new();

    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            other_conversation_id,
            Some(turn_id),
            9,
            "other conversation",
        ))
        .expect("put other conversation");
    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            conversation_id,
            Some(other_turn_id),
            8,
            "other turn",
        ))
        .expect("put other turn");
    store
        .put_checkpoint(checkpoint(
            expected_id,
            conversation_id,
            Some(turn_id),
            4,
            "expected",
        ))
        .expect("put expected");

    let latest = store
        .latest_checkpoint(conversation_id, Some(turn_id))
        .expect("load latest")
        .expect("checkpoint exists");

    assert_eq!(latest.checkpoint_id, expected_id);
    assert_eq!(latest.label, "expected");
}

fn assert_rejects_duplicate_checkpoint_id<S: CheckpointStore>(store: &mut S) {
    let checkpoint_id = CheckpointId::new();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .put_checkpoint(checkpoint(
            checkpoint_id,
            conversation_id,
            Some(turn_id),
            1,
            "original",
        ))
        .expect("put original");
    let error = store
        .put_checkpoint(checkpoint(
            checkpoint_id,
            conversation_id,
            Some(turn_id),
            2,
            "duplicate",
        ))
        .expect_err("duplicate rejected");

    assert!(matches!(
        error,
        StorageError::DuplicateCheckpoint(id) if id == checkpoint_id
    ));
}

fn assert_rejects_empty_label<S: CheckpointStore>(store: &mut S) {
    let error = store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            ConversationId::new(),
            Some(TurnId::new()),
            1,
            " ",
        ))
        .expect_err("empty label rejected");

    assert!(
        error
            .to_string()
            .contains("checkpoint label cannot be empty")
    );
}

#[test]
fn checkpoint_store_serializes_stable_shape() {
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let record = checkpoint(
        CheckpointId::new(),
        conversation_id,
        Some(turn_id),
        7,
        "stable",
    );

    let value = serde_json::to_value(&record).expect("serialize checkpoint");

    assert_eq!(value["conversation_id"], conversation_id.to_string());
    assert_eq!(value["turn_id"], turn_id.to_string());
    assert_eq!(value["event_seq"], 7);
    assert_eq!(value["label"], "stable");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["state_json"]["turn_state"], "observed");
}

#[test]
fn checkpoint_store_in_memory_puts_and_loads_latest_checkpoint() {
    let mut store = InMemoryEventStore::default();
    assert_puts_and_loads_latest_checkpoint(&mut store);
}

#[test]
fn checkpoint_store_sqlite_puts_and_loads_latest_checkpoint() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite");
    assert_puts_and_loads_latest_checkpoint(&mut store);
}

#[test]
fn checkpoint_store_in_memory_filters_by_conversation_and_turn() {
    let mut store = InMemoryEventStore::default();
    assert_filters_by_conversation_and_turn(&mut store);
}

#[test]
fn checkpoint_store_sqlite_filters_by_conversation_and_turn() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite");
    assert_filters_by_conversation_and_turn(&mut store);
}

#[test]
fn checkpoint_store_in_memory_rejects_duplicate_checkpoint_id() {
    let mut store = InMemoryEventStore::default();
    assert_rejects_duplicate_checkpoint_id(&mut store);
}

#[test]
fn checkpoint_store_sqlite_rejects_duplicate_checkpoint_id() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite");
    assert_rejects_duplicate_checkpoint_id(&mut store);
}

#[test]
fn checkpoint_store_in_memory_rejects_empty_label() {
    let mut store = InMemoryEventStore::default();
    assert_rejects_empty_label(&mut store);
}

#[test]
fn checkpoint_store_sqlite_rejects_empty_label() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite");
    assert_rejects_empty_label(&mut store);
}
