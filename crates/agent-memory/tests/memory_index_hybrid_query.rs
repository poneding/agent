use agent_memory::{
    EmbeddingMetadata, EmbeddingVector, HybridMemoryQuery, InMemoryMemoryIndex, MemoryError,
    MemoryIndex, MemoryRecord, MemoryScope, MemorySensitivity, MemorySource,
};
use agent_types::{ConversationId, WorkspaceId};

fn conversation_scope(value: &str) -> MemoryScope {
    MemoryScope::Conversation(value.parse::<ConversationId>().expect("conversation id"))
}

fn project_scope(value: &str) -> MemoryScope {
    MemoryScope::Project {
        workspace_id: value.parse::<WorkspaceId>().expect("workspace id"),
    }
}

fn record(
    id: &str,
    scope: MemoryScope,
    text: &str,
    sensitivity: MemorySensitivity,
) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        scope,
        text: text.to_string(),
        source: MemorySource {
            event_id: None,
            description: "test memory".to_string(),
        },
        confidence: 80,
        sensitivity,
    }
}

fn embedding(record_id: &str, values: Vec<f32>) -> EmbeddingVector {
    EmbeddingVector::new(
        record_id,
        values,
        EmbeddingMetadata::new(
            "fixture",
            "fixture-embedding",
            2,
            format!("{record_id}-hash"),
            MemorySensitivity::Internal,
        )
        .expect("metadata"),
    )
    .expect("embedding")
}

fn query(
    text: &str,
    query_vector: Option<Vec<f32>>,
    scopes: Vec<MemoryScope>,
    lexical_weight: f32,
    semantic_weight: f32,
    top_k: usize,
    minimum_score: f32,
) -> HybridMemoryQuery {
    HybridMemoryQuery::new(
        text,
        query_vector,
        scopes,
        MemorySensitivity::Internal,
        lexical_weight,
        semantic_weight,
        top_k,
        minimum_score,
        None,
    )
    .expect("query")
}

#[test]
fn lexical_query_filters_scope_sensitivity_and_orders_deterministically() {
    let conversation = conversation_scope("00000000-0000-0000-0000-000000002301");
    let other_conversation = conversation_scope("00000000-0000-0000-0000-000000002302");
    let mut index = InMemoryMemoryIndex::default();
    index
        .upsert_record(record(
            "memory-b",
            conversation.clone(),
            "Rust agent runtime memory retrieval",
            MemorySensitivity::Internal,
        ))
        .expect("upsert b");
    index
        .upsert_record(record(
            "memory-a",
            conversation.clone(),
            "Rust runtime runtime",
            MemorySensitivity::Internal,
        ))
        .expect("upsert a");
    index
        .upsert_record(record(
            "other-scope",
            other_conversation,
            "Rust runtime memory",
            MemorySensitivity::Internal,
        ))
        .expect("upsert other");
    index
        .upsert_record(record(
            "sensitive",
            conversation.clone(),
            "Rust runtime memory secret",
            MemorySensitivity::Sensitive,
        ))
        .expect("upsert sensitive");

    let results = index
        .query(query(
            "rust runtime memory",
            None,
            vec![conversation],
            1.0,
            0.0,
            10,
            0.0,
        ))
        .expect("query");
    let ids = results
        .iter()
        .map(|candidate| candidate.record.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["memory-b", "memory-a"]);
    assert_eq!(results[0].matched_terms, vec!["memory", "runtime", "rust"]);
    assert!(results[0].lexical_score > results[1].lexical_score);
    assert_eq!(results[0].semantic_score, None);
    assert!(results[0].combined_score <= 1.0);
}

#[test]
fn query_applies_top_k_and_minimum_score() {
    let scope = conversation_scope("00000000-0000-0000-0000-000000002303");
    let mut index = InMemoryMemoryIndex::default();
    index
        .upsert_record(record(
            "strong",
            scope.clone(),
            "alpha beta gamma",
            MemorySensitivity::Public,
        ))
        .expect("strong");
    index
        .upsert_record(record(
            "weak",
            scope.clone(),
            "alpha only",
            MemorySensitivity::Public,
        ))
        .expect("weak");
    index
        .upsert_record(record(
            "none",
            scope.clone(),
            "delta epsilon",
            MemorySensitivity::Public,
        ))
        .expect("none");

    let results = index
        .query(
            HybridMemoryQuery::new(
                "alpha beta gamma",
                None,
                vec![scope],
                MemorySensitivity::Public,
                1.0,
                0.0,
                1,
                0.5,
                Some(128),
            )
            .expect("query"),
        )
        .expect("query results");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].record.id, "strong");
    assert_eq!(results[0].combined_score, 1.0);
}

#[test]
fn semantic_query_uses_cosine_similarity_and_hybrid_weights() {
    let scope = project_scope("00000000-0000-0000-0000-000000002304");
    let mut index = InMemoryMemoryIndex::default();
    index
        .upsert_record(record(
            "semantic-best",
            scope.clone(),
            "unrelated lexical text",
            MemorySensitivity::Internal,
        ))
        .expect("best record");
    index
        .upsert_record(record(
            "lexical-best",
            scope.clone(),
            "rust runtime memory",
            MemorySensitivity::Internal,
        ))
        .expect("lexical record");
    index
        .upsert_embedding(embedding("semantic-best", vec![1.0, 0.0]))
        .expect("best embedding");
    index
        .upsert_embedding(embedding("lexical-best", vec![0.0, 1.0]))
        .expect("lexical embedding");

    let semantic_results = index
        .query(query(
            "rust runtime",
            Some(vec![1.0, 0.0]),
            vec![scope.clone()],
            0.2,
            0.8,
            10,
            0.0,
        ))
        .expect("semantic query");
    let lexical_results = index
        .query(query(
            "rust runtime",
            Some(vec![1.0, 0.0]),
            vec![scope],
            0.9,
            0.1,
            10,
            0.0,
        ))
        .expect("lexical query");

    assert_eq!(semantic_results[0].record.id, "semantic-best");
    assert_eq!(semantic_results[0].semantic_score, Some(1.0));
    assert_eq!(lexical_results[0].record.id, "lexical-best");
    assert!(lexical_results[0].lexical_score > 0.0);
}

#[test]
fn missing_embeddings_fall_back_to_lexical_scores() {
    let scope = conversation_scope("00000000-0000-0000-0000-000000002305");
    let mut index = InMemoryMemoryIndex::default();
    index
        .upsert_record(record(
            "memory",
            scope.clone(),
            "rust runtime memory",
            MemorySensitivity::Internal,
        ))
        .expect("record");

    let results = index
        .query(query(
            "rust runtime",
            Some(vec![1.0, 0.0]),
            vec![scope],
            1.0,
            1.0,
            10,
            0.0,
        ))
        .expect("query");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].record.id, "memory");
    assert_eq!(results[0].semantic_score, None);
    assert!(results[0].combined_score > 0.0);
}

#[test]
fn index_rejects_orphan_embedding() {
    let mut index = InMemoryMemoryIndex::default();

    let error = index
        .upsert_embedding(embedding("missing", vec![1.0, 0.0]))
        .expect_err("orphan rejected");

    assert!(matches!(error, MemoryError::InvalidEmbedding(message) if message.contains("orphan")));
}

#[test]
fn hybrid_query_rejects_invalid_parameters() {
    let scope = conversation_scope("00000000-0000-0000-0000-000000002306");
    let empty_query = HybridMemoryQuery::new(
        " ",
        None,
        vec![scope.clone()],
        MemorySensitivity::Internal,
        1.0,
        0.0,
        10,
        0.0,
        None,
    )
    .expect_err("empty query rejected");
    let empty_scopes = HybridMemoryQuery::new(
        "memory",
        None,
        Vec::new(),
        MemorySensitivity::Internal,
        1.0,
        0.0,
        10,
        0.0,
        None,
    )
    .expect_err("empty scopes rejected");
    let zero_top_k = HybridMemoryQuery::new(
        "memory",
        None,
        vec![scope.clone()],
        MemorySensitivity::Internal,
        1.0,
        0.0,
        0,
        0.0,
        None,
    )
    .expect_err("zero top k rejected");
    let invalid_weight = HybridMemoryQuery::new(
        "memory",
        None,
        vec![scope.clone()],
        MemorySensitivity::Internal,
        f32::NAN,
        0.0,
        10,
        0.0,
        None,
    )
    .expect_err("nan weight rejected");
    let invalid_vector = HybridMemoryQuery::new(
        "memory",
        Some(vec![1.0, f32::INFINITY]),
        vec![scope],
        MemorySensitivity::Internal,
        1.0,
        0.0,
        10,
        0.0,
        None,
    )
    .expect_err("non finite vector rejected");

    assert!(matches!(empty_query, MemoryError::InvalidQuery(message) if message.contains("query")));
    assert!(
        matches!(empty_scopes, MemoryError::InvalidQuery(message) if message.contains("scope"))
    );
    assert!(matches!(zero_top_k, MemoryError::InvalidQuery(message) if message.contains("top")));
    assert!(
        matches!(invalid_weight, MemoryError::InvalidQuery(message) if message.contains("weight"))
    );
    assert!(
        matches!(invalid_vector, MemoryError::InvalidQuery(message) if message.contains("finite"))
    );
}
