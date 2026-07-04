use agent_memory::{
    DeterministicMemoryReranker, MemoryPrivacyDiagnostic, MemoryPrivacyDiagnosticSeverity,
    MemoryProductSnapshot, MemoryRecord, MemoryRerankRequest, MemoryReranker,
    MemoryRetrievalCandidate, MemoryScope, MemorySensitivity, MemorySource, MemoryTombstone,
    MemoryWriteProposal,
};
use agent_types::{ConversationId, EventId, WorkspaceId};

fn event_id() -> EventId {
    "00000000-0000-0000-0000-000000006401"
        .parse::<EventId>()
        .expect("event id")
}

fn conversation_scope() -> MemoryScope {
    MemoryScope::Conversation(
        "00000000-0000-0000-0000-000000006402"
            .parse::<ConversationId>()
            .expect("conversation id"),
    )
}

fn project_scope() -> MemoryScope {
    MemoryScope::Project {
        workspace_id: "00000000-0000-0000-0000-000000006403"
            .parse::<WorkspaceId>()
            .expect("workspace id"),
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
            event_id: Some(event_id()),
            description: "memory product snapshot fixture".to_string(),
        },
        confidence: 91,
        sensitivity,
    }
}

fn proposal(
    proposal_id: &str,
    proposed_record_id: &str,
    scope: MemoryScope,
    sensitivity: MemorySensitivity,
) -> MemoryWriteProposal {
    MemoryWriteProposal::new(
        proposal_id,
        proposed_record_id,
        scope,
        "store raw sk-secret token content only behind review",
        "remember review preference for sk-secret token values",
        vec![event_id()],
        88,
        sensitivity,
    )
    .expect("valid proposal")
}

fn candidate(id: &str, text: &str, sensitivity: MemorySensitivity) -> MemoryRetrievalCandidate {
    MemoryRetrievalCandidate {
        record: record(id, project_scope(), text, sensitivity),
        lexical_score: 0.5,
        semantic_score: Some(0.4),
        combined_score: 0.9,
        matched_terms: vec!["memory".to_string(), "sk-secret".to_string()],
    }
}

#[test]
fn memory_product_snapshot_serializes_stable_empty_shape() {
    let snapshot = MemoryProductSnapshot::empty("project:agent");
    let json = serde_json::to_value(&snapshot).expect("serialize snapshot");

    assert_eq!(json["scope_label"], "project:agent");
    assert_eq!(json["records"], serde_json::json!([]));
    assert_eq!(json["proposals"], serde_json::json!([]));
    assert_eq!(json["retrieval"], serde_json::Value::Null);
    assert_eq!(json["influence"], serde_json::Value::Null);
    assert_eq!(json["tombstone_count"], 0);
    assert_eq!(json["diagnostics"], serde_json::json!([]));
    assert_eq!(json["retention"]["memory_enabled"], true);
}

#[test]
fn memory_product_snapshot_bounds_public_record_previews() {
    let long_text = format!(
        "prefer deterministic memory review workflows {}",
        "alpha ".repeat(80)
    );
    let snapshot = MemoryProductSnapshot::from_parts(
        "project:agent",
        &[record(
            "memory-public",
            project_scope(),
            &long_text,
            MemorySensitivity::Public,
        )],
        &[],
        None,
        None,
        Default::default(),
        &[],
        Vec::new(),
    );

    let summary = &snapshot.records[0];
    let preview = summary.text_preview.as_ref().expect("public preview");
    assert_eq!(summary.id, "memory-public");
    assert!(!summary.preview_redacted);
    assert!(summary.preview_truncated);
    assert!(preview.len() <= 123);
    assert!(preview.ends_with("..."));
}

#[test]
fn memory_product_snapshot_hides_sensitive_record_content_by_default() {
    let snapshot = MemoryProductSnapshot::from_parts(
        "project:agent",
        &[record(
            "memory-sensitive",
            project_scope(),
            "private sk-secret password should not appear",
            MemorySensitivity::Sensitive,
        )],
        &[],
        None,
        None,
        Default::default(),
        &[],
        Vec::new(),
    );

    let summary = &snapshot.records[0];
    assert_eq!(summary.text_preview.as_deref(), Some("[redacted]"));
    assert!(summary.preview_redacted);

    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");
    assert!(!serialized.contains("private sk-secret password"));
    assert!(!serialized.contains("password should not appear"));
}

#[test]
fn memory_product_snapshot_summarizes_proposal_review_state() {
    let snapshot = MemoryProductSnapshot::from_parts(
        "project:agent",
        &[],
        &[proposal(
            "proposal-safe",
            "record-safe",
            conversation_scope(),
            MemorySensitivity::Sensitive,
        )],
        None,
        None,
        Default::default(),
        &[],
        Vec::new(),
    );

    let summary = &snapshot.proposals[0];
    assert_eq!(summary.proposal_id, "proposal-safe");
    assert_eq!(summary.source_event_count, 1);
    assert!(summary.requires_explicit_approval);
    assert!(summary.content_redacted);
    assert!(!summary.auto_approval.eligible);

    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("store raw sk-secret token content"));
    assert!(!serialized.contains("sk-secret"));
}

#[test]
fn memory_product_snapshot_summarizes_retrieval_counts_and_safe_reasons() {
    let result = DeterministicMemoryReranker::default()
        .rerank(MemoryRerankRequest {
            query_text: "memory preference".to_string(),
            candidates: vec![
                candidate(
                    "selected-memory",
                    "selected public memory preference",
                    MemorySensitivity::Public,
                ),
                candidate(
                    "rejected-memory",
                    "rejected sensitive sk-secret memory",
                    MemorySensitivity::Sensitive,
                ),
            ],
            max_selected: 1,
            prompt_budget_tokens: None,
        })
        .expect("rerank result");
    let snapshot = MemoryProductSnapshot::from_parts(
        "project:agent",
        &[],
        &[],
        Some(&result),
        None,
        Default::default(),
        &[],
        Vec::new(),
    );

    let retrieval = snapshot.retrieval.as_ref().expect("retrieval preview");
    assert_eq!(retrieval.selected_count, 1);
    assert_eq!(retrieval.rejected_count, 1);
    assert_eq!(retrieval.candidates.len(), 2);
    assert_eq!(retrieval.candidates[0].memory_id, "selected-memory");
    assert!(retrieval.candidates[0].selected);
    assert!(!retrieval.candidates[1].selected);
    assert_eq!(
        retrieval.candidates[1].preview.as_deref(),
        Some("[redacted]")
    );

    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");
    assert!(serialized.contains("\"score\""));
    assert!(!serialized.contains("sk-secret"));
}

#[test]
fn memory_product_snapshot_redacts_influence_trace_terms_and_previews() {
    let result = DeterministicMemoryReranker::default()
        .rerank(MemoryRerankRequest {
            query_text: "memory preference".to_string(),
            candidates: vec![candidate(
                "sensitive-memory",
                "sensitive sk-secret token preference",
                MemorySensitivity::Sensitive,
            )],
            max_selected: 1,
            prompt_budget_tokens: None,
        })
        .expect("rerank result");
    let snapshot = MemoryProductSnapshot::from_parts(
        "project:agent",
        &[],
        &[],
        None,
        Some(&result.trace),
        Default::default(),
        &[],
        Vec::new(),
    );

    let influence = snapshot.influence.as_ref().expect("influence summary");
    assert_eq!(influence.selected_count, 1);
    assert_eq!(influence.entries[0].preview.as_deref(), Some("[redacted]"));
    assert_eq!(
        influence.entries[0].matched_terms,
        vec!["memory".to_string(), "[redacted]".to_string()]
    );

    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");
    assert!(!serialized.contains("sensitive sk-secret token"));
    assert!(!serialized.contains("sk-secret"));
}

#[test]
fn memory_product_snapshot_excludes_tombstoned_records_from_active_summaries() {
    let scope = project_scope();
    let active = record(
        "active-memory",
        scope.clone(),
        "active project memory",
        MemorySensitivity::Public,
    );
    let deleted = record(
        "deleted-memory",
        scope.clone(),
        "deleted project memory",
        MemorySensitivity::Public,
    );
    let tombstone = MemoryTombstone::new(scope, "deleted-memory", "user removed token", None);

    let snapshot = MemoryProductSnapshot::from_parts(
        "project:agent",
        &[active, deleted],
        &[],
        None,
        None,
        Default::default(),
        &[tombstone],
        Vec::new(),
    );

    let ids = snapshot
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["active-memory"]);
    assert_eq!(snapshot.tombstone_count, 1);
}

#[test]
fn memory_product_snapshot_bounds_and_redacts_privacy_diagnostics() {
    let diagnostic = MemoryPrivacyDiagnostic::new(
        "secret-token-diagnostic",
        MemoryPrivacyDiagnosticSeverity::Warning,
        format!(
            "{} {}",
            "sk-secret token ".repeat(20),
            "should stay private"
        ),
    );
    let snapshot = MemoryProductSnapshot::from_parts(
        "project:agent",
        &[],
        &[],
        None,
        None,
        Default::default(),
        &[],
        vec![diagnostic],
    );

    let diagnostic = &snapshot.diagnostics[0];
    assert_eq!(diagnostic.code, "[redacted]");
    assert!(diagnostic.message.len() <= 163);
    assert!(diagnostic.message.ends_with("..."));

    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("should stay private"));
}
