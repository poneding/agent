use agent_storage::{
    IdempotencyKey, IdempotencyStore, InMemoryEventStore, SqliteEventStore, StorageError,
    ToolEffectRecord,
};
use agent_types::Timestamp;
use serde_json::json;

fn effect(key: IdempotencyKey, input_hash: &str, output: &str) -> ToolEffectRecord {
    ToolEffectRecord {
        key,
        call_id: "call-1".to_string(),
        tool_name: "workspace.edit_file".to_string(),
        input_hash: input_hash.to_string(),
        output_json: json!({
            "applied": true,
            "output": output
        }),
        completed_at: Timestamp::now_utc(),
    }
}

fn assert_records_and_gets_effect<S: IdempotencyStore>(store: &mut S) {
    let key = IdempotencyKey::new("turn-1:call-1").expect("valid key");
    let record = effect(key.clone(), "sha256:abc", "first");

    store.record_effect(record.clone()).expect("record effect");

    let loaded = store
        .get_effect(&key)
        .expect("get effect")
        .expect("effect exists");

    assert_eq!(loaded, record);
}

fn assert_accepts_duplicate_same_hash<S: IdempotencyStore>(store: &mut S) {
    let key = IdempotencyKey::new("turn-1:call-1").expect("valid key");
    let original = effect(key.clone(), "sha256:abc", "first");
    let duplicate = effect(key.clone(), "sha256:abc", "first");

    store
        .record_effect(original.clone())
        .expect("record original");
    store
        .record_effect(duplicate)
        .expect("same hash duplicate is idempotent");

    let loaded = store
        .get_effect(&key)
        .expect("get effect")
        .expect("effect exists");

    assert_eq!(loaded, original);
}

fn assert_rejects_duplicate_mismatched_hash<S: IdempotencyStore>(store: &mut S) {
    let key = IdempotencyKey::new("turn-1:call-1").expect("valid key");

    store
        .record_effect(effect(key.clone(), "sha256:abc", "first"))
        .expect("record original");
    let error = store
        .record_effect(effect(key, "sha256:different", "second"))
        .expect_err("mismatched hash rejected");

    assert!(matches!(error, StorageError::IdempotencyConflict { .. }));
}

#[test]
fn idempotency_store_key_validates_and_serializes() {
    let key = IdempotencyKey::new("turn-1:call-1").expect("valid key");
    let json = serde_json::to_string(&key).expect("serialize key");
    let parsed: IdempotencyKey = serde_json::from_str(&json).expect("deserialize key");

    assert_eq!(key.as_str(), "turn-1:call-1");
    assert_eq!(json, "\"turn-1:call-1\"");
    assert_eq!(parsed, key);
}

#[test]
fn idempotency_store_key_rejects_empty_or_whitespace() {
    let error = IdempotencyKey::new("  ").expect_err("empty key rejected");

    assert!(
        error
            .to_string()
            .contains("idempotency key cannot be empty")
    );
}

#[test]
fn idempotency_store_in_memory_records_and_gets_effect() {
    let mut store = InMemoryEventStore::default();
    assert_records_and_gets_effect(&mut store);
}

#[test]
fn idempotency_store_sqlite_records_and_gets_effect() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite");
    assert_records_and_gets_effect(&mut store);
}

#[test]
fn idempotency_store_in_memory_accepts_duplicate_same_hash() {
    let mut store = InMemoryEventStore::default();
    assert_accepts_duplicate_same_hash(&mut store);
}

#[test]
fn idempotency_store_sqlite_accepts_duplicate_same_hash() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite");
    assert_accepts_duplicate_same_hash(&mut store);
}

#[test]
fn idempotency_store_in_memory_rejects_duplicate_mismatched_hash() {
    let mut store = InMemoryEventStore::default();
    assert_rejects_duplicate_mismatched_hash(&mut store);
}

#[test]
fn idempotency_store_sqlite_rejects_duplicate_mismatched_hash() {
    let mut store = SqliteEventStore::open_in_memory().expect("open sqlite");
    assert_rejects_duplicate_mismatched_hash(&mut store);
}
