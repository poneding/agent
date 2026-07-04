use agent_memory::{
    DeterministicMemoryReranker, HybridMemoryQuery, InMemoryMemoryEditor, InMemoryMemoryIndex,
    InMemoryMemoryProposalStore, MemoryArchive, MemoryArchiveError, MemoryArchiveManifest,
    MemoryArchiveRecord, MemoryEditor, MemoryImportPolicy, MemoryIndex, MemoryProposalDecision,
    MemoryProposalStore, MemoryRecord, MemoryRerankRequest, MemoryReranker, MemoryRetentionPolicy,
    MemoryScope, MemorySensitivity, MemorySource, MemoryTombstoneConflictPolicy,
    MemoryWriteProposal,
};
use agent_types::{ConversationId, EventId, WorkspaceId};

fn event_id() -> EventId {
    "00000000-0000-0000-0000-000000002505"
        .parse::<EventId>()
        .expect("event id")
}

fn conversation_scope() -> MemoryScope {
    MemoryScope::Conversation(
        "00000000-0000-0000-0000-000000002515"
            .parse::<ConversationId>()
            .expect("conversation id"),
    )
}

fn project_scope() -> MemoryScope {
    MemoryScope::Project {
        workspace_id: "00000000-0000-0000-0000-000000002525"
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
            description: "memory editor fixture".to_string(),
        },
        confidence: 90,
        sensitivity,
    }
}

#[test]
fn memory_editor_portability_deletes_with_tombstone_and_repeats_idempotently() {
    let scope = conversation_scope();
    let mut editor = InMemoryMemoryEditor::default();
    editor
        .add_record(record(
            "memory-a",
            scope.clone(),
            "remember shell approval preference",
            MemorySensitivity::Internal,
        ))
        .expect("add record");

    let tombstone = editor
        .delete_memory(
            &scope,
            "memory-a",
            "remove sk-secret token",
            Some(event_id()),
        )
        .expect("delete memory");
    let repeated = editor
        .delete_memory(&scope, "memory-a", "different reason", Some(event_id()))
        .expect("repeat delete");

    assert_eq!(repeated, tombstone);
    assert_eq!(tombstone.record_id, "memory-a");
    assert_eq!(tombstone.reason, "remove [redacted] [redacted]");
    assert!(editor.list_summaries(&scope).expect("summaries").is_empty());
    assert_eq!(editor.tombstones().len(), 1);
}

#[test]
fn memory_editor_portability_rebuild_skips_tombstones_and_removes_stale_embeddings() {
    let scope = project_scope();
    let mut editor = InMemoryMemoryEditor::default();
    editor
        .add_record(record(
            "active",
            scope.clone(),
            "active memory survives rebuild",
            MemorySensitivity::Public,
        ))
        .expect("active");
    editor
        .add_record(record(
            "deleted",
            scope.clone(),
            "deleted memory should stay gone",
            MemorySensitivity::Public,
        ))
        .expect("deleted");
    editor
        .upsert_embedding_marker("active")
        .expect("active embedding");
    editor
        .upsert_embedding_marker("deleted")
        .expect("deleted embedding");
    editor
        .upsert_embedding_marker("orphan")
        .expect("orphan embedding");
    editor
        .delete_memory(&scope, "deleted", "user removed it", None)
        .expect("delete");

    let report = editor.rebuild_index().expect("rebuild");

    assert_eq!(report.indexed_records, 1);
    assert_eq!(report.skipped_tombstoned, 1);
    assert_eq!(report.stale_embeddings_removed, 2);
    assert_eq!(editor.embedding_marker_count(), 1);
}

#[test]
fn memory_editor_portability_export_preserves_tombstones_and_omits_sensitive_by_default() {
    let scope = project_scope();
    let mut editor = InMemoryMemoryEditor::default();
    editor
        .add_record(record(
            "public-memory",
            scope.clone(),
            "public project memory",
            MemorySensitivity::Public,
        ))
        .expect("public");
    editor
        .add_record(record(
            "sensitive-memory",
            scope.clone(),
            "private sensitive memory",
            MemorySensitivity::Sensitive,
        ))
        .expect("sensitive");
    editor
        .add_record(record(
            "deleted-memory",
            scope.clone(),
            "deleted public memory",
            MemorySensitivity::Public,
        ))
        .expect("deleted");
    editor
        .delete_memory(&scope, "deleted-memory", "cleanup", None)
        .expect("delete");

    let archive = editor
        .export_archive(MemoryRetentionPolicy {
            include_sensitive: false,
            max_records: None,
        })
        .expect("export");

    assert_eq!(archive.records.len(), 1);
    assert_eq!(archive.records[0].id, "public-memory");
    assert_eq!(archive.tombstones.len(), 1);
    assert_eq!(archive.tombstones[0].record_id, "deleted-memory");

    let serialized = serde_json::to_string(&archive).expect("serialize archive");
    assert!(!serialized.contains("private sensitive memory"));
    assert!(!serialized.contains("embedding"));
}

#[test]
fn memory_editor_portability_import_skips_or_rejects_tombstone_conflicts() {
    let scope = conversation_scope();
    let archive = MemoryArchive {
        schema_version: 1,
        manifest: MemoryArchiveManifest {
            exported_by: "agent-memory".to_string(),
            record_count: 2,
            sensitive_record_count: 0,
        },
        records: vec![
            MemoryArchiveRecord::from(record(
                "deleted-memory",
                scope.clone(),
                "should not return",
                MemorySensitivity::Public,
            )),
            MemoryArchiveRecord::from(record(
                "fresh-memory",
                scope.clone(),
                "fresh imported memory",
                MemorySensitivity::Public,
            )),
        ],
        tombstones: Vec::new(),
    };

    let mut skip_editor = InMemoryMemoryEditor::default();
    skip_editor
        .add_record(record(
            "deleted-memory",
            scope.clone(),
            "local deleted memory",
            MemorySensitivity::Public,
        ))
        .expect("local record");
    skip_editor
        .delete_memory(&scope, "deleted-memory", "local delete", None)
        .expect("delete");

    let outcome = skip_editor
        .import_archive(
            &archive,
            MemoryImportPolicy {
                tombstone_conflicts: MemoryTombstoneConflictPolicy::Skip,
            },
        )
        .expect("skip import");
    assert_eq!(outcome.report.accepted, 1);
    assert_eq!(outcome.report.skipped, 1);
    assert_eq!(
        skip_editor
            .inspect_summary(&scope, "fresh-memory")
            .expect("fresh summary")
            .id,
        "fresh-memory"
    );
    assert!(
        skip_editor
            .inspect_summary(&scope, "deleted-memory")
            .is_none()
    );

    let mut reject_editor = InMemoryMemoryEditor::default();
    reject_editor
        .add_record(record(
            "deleted-memory",
            scope.clone(),
            "local deleted memory",
            MemorySensitivity::Public,
        ))
        .expect("local record");
    reject_editor
        .delete_memory(&scope, "deleted-memory", "local delete", None)
        .expect("delete");
    let error = reject_editor
        .import_archive(
            &archive,
            MemoryImportPolicy {
                tombstone_conflicts: MemoryTombstoneConflictPolicy::Reject,
            },
        )
        .expect_err("reject import");
    assert!(matches!(
        error,
        MemoryArchiveError::TombstoneConflict { id } if id == "deleted-memory"
    ));
    assert!(
        reject_editor
            .list_summaries(&scope)
            .expect("list")
            .is_empty()
    );
}

#[test]
fn memory_editor_portability_corrupt_archive_fails_without_mutation() {
    let scope = conversation_scope();
    let mut editor = InMemoryMemoryEditor::default();
    editor
        .add_record(record(
            "existing",
            scope.clone(),
            "existing memory",
            MemorySensitivity::Public,
        ))
        .expect("existing");
    let corrupt = MemoryArchive {
        schema_version: 1,
        manifest: MemoryArchiveManifest {
            exported_by: "agent-memory".to_string(),
            record_count: 2,
            sensitive_record_count: 0,
        },
        records: vec![MemoryArchiveRecord::from(record(
            "new",
            scope.clone(),
            "new memory",
            MemorySensitivity::Public,
        ))],
        tombstones: Vec::new(),
    };

    let error = editor
        .import_archive(&corrupt, MemoryImportPolicy::default())
        .expect_err("manifest mismatch");

    assert!(matches!(error, MemoryArchiveError::ManifestMismatch));
    assert_eq!(editor.list_summaries(&scope).expect("list").len(), 1);
    assert!(editor.inspect_summary(&scope, "new").is_none());
}

#[test]
fn memory_editor_portability_summaries_redact_sensitive_and_secret_like_text() {
    let scope = conversation_scope();
    let mut editor = InMemoryMemoryEditor::default();
    editor
        .add_record(record(
            "public-secret-like",
            scope.clone(),
            "public memory mentions sk-secret token",
            MemorySensitivity::Public,
        ))
        .expect("public");
    editor
        .add_record(record(
            "sensitive-memory",
            scope.clone(),
            "sensitive raw text",
            MemorySensitivity::Sensitive,
        ))
        .expect("sensitive");

    let summaries = editor.list_summaries(&scope).expect("summaries");
    let public = summaries
        .iter()
        .find(|summary| summary.id == "public-secret-like")
        .expect("public summary");
    let sensitive = summaries
        .iter()
        .find(|summary| summary.id == "sensitive-memory")
        .expect("sensitive summary");

    assert_eq!(
        public.text_preview.as_deref(),
        Some("public memory mentions [redacted] [redacted]")
    );
    assert_eq!(sensitive.text_preview, None);

    let serialized = serde_json::to_string(&summaries).expect("serialize summaries");
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("sensitive raw text"));
}

#[test]
fn memory_editor_portability_final_m23_smoke_keeps_deleted_memory_out_of_retrieval() {
    let scope = project_scope();
    let mut proposals = InMemoryMemoryProposalStore::default();
    proposals
        .add_proposal(
            MemoryWriteProposal::new(
                "proposal",
                "learned-memory",
                scope.clone(),
                "rust agent memory retrieval preference",
                "retrieval preference",
                vec![event_id()],
                90,
                MemorySensitivity::Public,
            )
            .expect("proposal"),
        )
        .expect("add proposal");
    proposals
        .resolve("proposal", MemoryProposalDecision::Approve)
        .expect("approve");
    let record = proposals.commit("proposal").expect("commit");

    let mut editor = InMemoryMemoryEditor::default();
    editor.add_record(record).expect("editor add");
    let mut index = InMemoryMemoryIndex::default();
    for record in editor.active_records() {
        index.upsert_record(record).expect("index record");
    }
    let query = HybridMemoryQuery::new(
        "rust retrieval preference",
        None,
        vec![scope.clone()],
        MemorySensitivity::Public,
        1.0,
        0.0,
        5,
        0.0,
        None,
    )
    .expect("query");
    let candidates = index.query(query).expect("query result");
    let reranked = DeterministicMemoryReranker::default()
        .rerank(MemoryRerankRequest {
            query_text: "rust retrieval preference".to_string(),
            candidates,
            max_selected: 1,
            prompt_budget_tokens: Some(8),
        })
        .expect("rerank");
    assert_eq!(reranked.selected[0].candidate.record.id, "learned-memory");

    editor
        .delete_memory(&scope, "learned-memory", "user removed", None)
        .expect("delete");
    let rebuild = editor.rebuild_index().expect("rebuild");
    assert_eq!(rebuild.indexed_records, 0);
    assert!(editor.active_records().is_empty());
}
