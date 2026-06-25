use agent_memory::{
    InMemoryMemoryStore, MemoryError, MemoryRecord, MemoryScope, MemorySensitivity, MemorySource,
    MemoryStore,
};
use agent_types::ConversationId;
use serde_json::json;

#[test]
fn memory_sensitivity_uses_snake_case_json() {
    assert_eq!(
        serde_json::to_string(&MemorySensitivity::Internal).unwrap(),
        "\"internal\""
    );
}

fn conversation_id(value: &str) -> ConversationId {
    value.parse().expect("valid conversation id")
}

fn record(id: &str, scope: MemoryScope, text: &str, confidence: u8) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        scope,
        text: text.to_string(),
        source: MemorySource {
            event_id: None,
            description: "user correction".to_string(),
        },
        confidence,
        sensitivity: MemorySensitivity::Internal,
    }
}

#[test]
fn conversation_memory_adds_and_lists_records_by_scope() {
    let scope = MemoryScope::Conversation(ConversationId::new());
    let mut store = InMemoryMemoryStore::default();

    store
        .add(record("memory-b", scope.clone(), "second fact", 80))
        .expect("add second");
    store
        .add(record("memory-a", scope.clone(), "first fact", 80))
        .expect("add first");
    let records = store.list(&scope).expect("list records");
    let ids: Vec<_> = records.iter().map(|record| record.id.as_str()).collect();

    assert_eq!(ids, vec!["memory-a", "memory-b"]);
}

#[test]
fn conversation_memory_isolates_conversations() {
    let first_scope = MemoryScope::Conversation(ConversationId::new());
    let second_scope = MemoryScope::Conversation(ConversationId::new());
    let mut store = InMemoryMemoryStore::default();

    store
        .add(record(
            "memory",
            first_scope.clone(),
            "first conversation",
            80,
        ))
        .expect("add first");
    store
        .add(record(
            "memory",
            second_scope.clone(),
            "second conversation",
            80,
        ))
        .expect("add second");

    assert_eq!(
        store.list(&first_scope).expect("list first")[0].text,
        "first conversation"
    );
    assert_eq!(
        store.list(&second_scope).expect("list second")[0].text,
        "second conversation"
    );
}

#[test]
fn conversation_memory_deletes_records() {
    let scope = MemoryScope::Conversation(ConversationId::new());
    let mut store = InMemoryMemoryStore::default();

    store
        .add(record("memory", scope.clone(), "remember this", 80))
        .expect("add memory");

    assert!(store.delete(&scope, "memory").expect("delete existing"));
    assert!(store.list(&scope).expect("list records").is_empty());
    assert!(!store.delete(&scope, "memory").expect("delete missing"));
}

#[test]
fn conversation_memory_rejects_invalid_records() {
    let scope = MemoryScope::Conversation(ConversationId::new());
    let mut store = InMemoryMemoryStore::default();

    let empty_text = store
        .add(record("memory", scope.clone(), " ", 80))
        .expect_err("empty text rejected");
    let invalid_confidence = store
        .add(record("memory", scope, "valid text", 0))
        .expect_err("invalid confidence rejected");

    assert!(matches!(
        empty_text,
        MemoryError::InvalidRecord(message) if message.contains("text")
    ));
    assert!(matches!(
        invalid_confidence,
        MemoryError::InvalidRecord(message) if message.contains("confidence")
    ));
}

#[test]
fn memory_record_serializes_stable_shape() {
    let conversation_id = conversation_id("00000000-0000-0000-0000-000000000301");
    let value = serde_json::to_value(record(
        "memory",
        MemoryScope::Conversation(conversation_id),
        "remember this",
        80,
    ))
    .expect("serialize memory");

    assert_eq!(
        value,
        json!({
            "id": "memory",
            "scope": {
                "type": "conversation",
                "data": "00000000-0000-0000-0000-000000000301"
            },
            "text": "remember this",
            "source": {
                "event_id": null,
                "description": "user correction"
            },
            "confidence": 80,
            "sensitivity": "internal"
        })
    );
}
