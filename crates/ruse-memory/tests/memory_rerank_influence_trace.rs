use ruse_memory::{
    DeterministicMemoryReranker, MemoryError, MemoryRecord, MemoryRerankRequest, MemoryReranker,
    MemoryRetrievalCandidate, MemoryRetrievalReason, MemoryScope, MemorySensitivity, MemorySource,
};
use ruse_types::ConversationId;

fn conversation_scope() -> MemoryScope {
    MemoryScope::Conversation(
        "00000000-0000-0000-0000-000000002303"
            .parse::<ConversationId>()
            .expect("conversation id"),
    )
}

fn record(id: &str, text: &str, sensitivity: MemorySensitivity) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        scope: conversation_scope(),
        text: text.to_string(),
        source: MemorySource {
            event_id: None,
            description: "test memory".to_string(),
        },
        confidence: 80,
        sensitivity,
    }
}

fn candidate(id: &str, text: &str, combined_score: f32) -> MemoryRetrievalCandidate {
    MemoryRetrievalCandidate {
        record: record(id, text, MemorySensitivity::Internal),
        lexical_score: combined_score / 2.0,
        semantic_score: Some(combined_score / 2.0),
        combined_score,
        matched_terms: vec!["rust".to_string()],
    }
}

fn sensitive_candidate(id: &str, text: &str, combined_score: f32) -> MemoryRetrievalCandidate {
    MemoryRetrievalCandidate {
        record: record(id, text, MemorySensitivity::Sensitive),
        lexical_score: combined_score,
        semantic_score: None,
        combined_score,
        matched_terms: vec!["secret".to_string()],
    }
}

fn request(
    candidates: Vec<MemoryRetrievalCandidate>,
    max_selected: usize,
    prompt_budget_tokens: Option<u32>,
) -> MemoryRerankRequest {
    MemoryRerankRequest {
        query_text: "rust private query".to_string(),
        candidates,
        max_selected,
        prompt_budget_tokens,
    }
}

#[test]
fn memory_rerank_influence_trace_selects_by_score_and_records_rank_rejections() {
    let result = DeterministicMemoryReranker::default()
        .rerank(request(
            vec![
                candidate("second", "rust runtime memory", 0.7),
                candidate("best", "rust agent memory", 0.9),
                candidate("third", "rust fallback memory", 0.3),
            ],
            2,
            None,
        ))
        .expect("rerank result");

    let selected_ids = result
        .selected
        .iter()
        .map(|candidate| candidate.candidate.record.id.as_str())
        .collect::<Vec<_>>();
    let rejected_ids = result
        .rejected
        .iter()
        .map(|candidate| candidate.candidate.record.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(selected_ids, vec!["best", "second"]);
    assert_eq!(rejected_ids, vec!["third"]);
    assert_eq!(result.selected[0].rank, 1);
    assert_eq!(
        result.selected[0].reasons,
        vec![MemoryRetrievalReason::Score]
    );
    assert_eq!(
        result.rejected[0].reasons,
        vec![MemoryRetrievalReason::Score]
    );
    assert_eq!(result.trace.selected_count, 2);
    assert_eq!(result.trace.rejected_count, 1);
}

#[test]
fn memory_rerank_influence_trace_keeps_best_duplicate_and_rejects_lower_duplicate() {
    let result = DeterministicMemoryReranker::default()
        .rerank(request(
            vec![
                candidate("same", "older duplicate memory", 0.4),
                candidate("other", "other useful memory", 0.5),
                candidate("same", "newer duplicate memory", 0.8),
            ],
            3,
            None,
        ))
        .expect("rerank result");

    assert_eq!(result.selected.len(), 2);
    assert_eq!(result.selected[0].candidate.record.id, "same");
    assert_eq!(
        result.selected[0].candidate.record.text,
        "newer duplicate memory"
    );
    assert_eq!(result.rejected.len(), 1);
    assert_eq!(
        result.rejected[0].candidate.record.text,
        "older duplicate memory"
    );
    assert_eq!(
        result.rejected[0].reasons,
        vec![MemoryRetrievalReason::Duplicate]
    );
}

#[test]
fn memory_rerank_influence_trace_applies_prompt_budget_and_marks_overflow() {
    let result = DeterministicMemoryReranker::default()
        .rerank(request(
            vec![
                candidate("fits-one", "alpha beta", 0.9),
                candidate("fits-two", "gamma delta epsilon", 0.8),
                candidate("overflow", "zeta eta theta iota", 0.7),
            ],
            3,
            Some(5),
        ))
        .expect("rerank result");

    let selected_ids = result
        .selected
        .iter()
        .map(|candidate| candidate.candidate.record.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(selected_ids, vec!["fits-one", "fits-two"]);
    assert_eq!(result.selected[0].token_estimate, 2);
    assert_eq!(result.selected[1].token_estimate, 3);
    assert_eq!(result.rejected[0].candidate.record.id, "overflow");
    assert_eq!(
        result.rejected[0].reasons,
        vec![MemoryRetrievalReason::Budget]
    );
}

#[test]
fn memory_rerank_influence_trace_is_query_hashed_and_redacts_sensitive_text() {
    let reranker = DeterministicMemoryReranker::default();
    let first = reranker
        .rerank(request(
            vec![sensitive_candidate(
                "secret-memory",
                "sk-secret password should never appear",
                1.0,
            )],
            1,
            None,
        ))
        .expect("first trace");
    let second = reranker
        .rerank(request(
            vec![sensitive_candidate(
                "secret-memory",
                "sk-secret password should never appear",
                1.0,
            )],
            1,
            None,
        ))
        .expect("second trace");

    assert_eq!(first.trace.query_hash, second.trace.query_hash);
    assert!(!first.trace.query_hash.contains("rust private query"));
    assert_eq!(
        first.trace.entries[0].preview.as_deref(),
        Some("[redacted]")
    );
    assert_eq!(
        first.trace.entries[0].sensitivity,
        MemorySensitivity::Sensitive
    );
    assert_eq!(
        first.trace.entries[0].reasons,
        vec![MemoryRetrievalReason::Score]
    );

    let serialized = serde_json::to_string(&first.trace).expect("serialize trace");
    assert!(serialized.contains("\"score\""));
    assert!(!serialized.contains("rust private query"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("password should never appear"));
}

#[test]
fn memory_rerank_influence_trace_returns_empty_result_for_empty_candidates() {
    let result = DeterministicMemoryReranker::default()
        .rerank(request(Vec::new(), 3, Some(64)))
        .expect("empty result");

    assert!(result.selected.is_empty());
    assert!(result.rejected.is_empty());
    assert_eq!(result.trace.selected_count, 0);
    assert_eq!(result.trace.rejected_count, 0);
}

#[test]
fn memory_rerank_influence_trace_rejects_invalid_limits_and_scores() {
    let invalid_limit = DeterministicMemoryReranker::default()
        .rerank(request(
            vec![candidate("memory", "valid memory", 0.5)],
            0,
            None,
        ))
        .expect_err("invalid max selected rejected");
    assert!(matches!(
        invalid_limit,
        MemoryError::InvalidRerank(message) if message.contains("max_selected")
    ));

    let non_finite = DeterministicMemoryReranker::default()
        .rerank(request(
            vec![MemoryRetrievalCandidate {
                combined_score: f32::NAN,
                ..candidate("memory", "valid memory", 0.5)
            }],
            1,
            None,
        ))
        .expect_err("non-finite scores rejected");
    assert!(matches!(
        non_finite,
        MemoryError::InvalidRerank(message) if message.contains("finite")
    ));
}
