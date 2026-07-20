use ruse_memory::{
    MemoryArchive, MemoryArchiveError, MemoryArchiveManifest, MemoryArchiveRecord,
    MemoryImportReport, MemoryRetentionPolicy, MemoryScope, MemorySensitivity, MemorySource,
    export_memory_archive, import_memory_archive,
};
use ruse_types::{ConversationId, WorkspaceId};
use serde_json::json;

fn conversation_id(value: &str) -> ConversationId {
    value.parse().expect("valid conversation id")
}

fn workspace_id(value: &str) -> WorkspaceId {
    value.parse().expect("valid workspace id")
}

fn record(
    id: &str,
    scope: MemoryScope,
    text: &str,
    sensitivity: MemorySensitivity,
) -> ruse_memory::MemoryRecord {
    ruse_memory::MemoryRecord {
        id: id.to_string(),
        scope,
        text: text.to_string(),
        source: MemorySource {
            event_id: None,
            description: "test fixture".to_string(),
        },
        confidence: 80,
        sensitivity,
    }
}

#[test]
fn memory_archive_serializes_stable_shape() {
    let scope = MemoryScope::Conversation(conversation_id("00000000-0000-0000-0000-000000000401"));
    let archive = export_memory_archive(
        &[record(
            "memory-a",
            scope,
            "remember project preference",
            MemorySensitivity::Sensitive,
        )],
        MemoryRetentionPolicy::default(),
    )
    .expect("export archive");

    assert_eq!(
        serde_json::to_value(archive).expect("serialize archive"),
        json!({
            "schema_version": 1,
            "manifest": {
                "exported_by": "ruse-memory",
                "record_count": 1,
                "sensitive_record_count": 1
            },
            "records": [{
                "id": "memory-a",
                "scope": {
                    "type": "conversation",
                    "data": "00000000-0000-0000-0000-000000000401"
                },
                "text": "remember project preference",
                "source": {
                    "event_id": null,
                    "description": "test fixture"
                },
                "confidence": 80,
                "sensitivity": "sensitive"
            }]
        })
    );
}

#[test]
fn memory_archive_export_applies_retention_policy() {
    let scope = MemoryScope::Conversation(ConversationId::new());
    let archive = export_memory_archive(
        &[
            record(
                "memory-b",
                scope.clone(),
                "sensitive note",
                MemorySensitivity::Sensitive,
            ),
            record(
                "memory-a",
                scope.clone(),
                "public note",
                MemorySensitivity::Public,
            ),
            record(
                "memory-c",
                scope,
                "internal note",
                MemorySensitivity::Internal,
            ),
        ],
        MemoryRetentionPolicy {
            include_sensitive: false,
            max_records: Some(1),
        },
    )
    .expect("export archive");

    assert_eq!(archive.records.len(), 1);
    assert_eq!(archive.records[0].id, "memory-a");
    assert_eq!(archive.manifest.record_count, 1);
    assert_eq!(archive.manifest.sensitive_record_count, 0);
}

#[test]
fn memory_archive_import_rejects_duplicate_ids() {
    let scope = MemoryScope::Conversation(ConversationId::new());
    let duplicate = MemoryArchive {
        schema_version: 1,
        manifest: MemoryArchiveManifest {
            exported_by: "ruse-memory".to_string(),
            record_count: 2,
            sensitive_record_count: 0,
        },
        records: vec![
            MemoryArchiveRecord::from(record(
                "memory",
                scope.clone(),
                "first",
                MemorySensitivity::Internal,
            )),
            MemoryArchiveRecord::from(record(
                "memory",
                scope,
                "second",
                MemorySensitivity::Internal,
            )),
        ],
        tombstones: Vec::new(),
    };

    let error = import_memory_archive(&duplicate).expect_err("duplicate ids rejected");

    assert!(matches!(
        error,
        MemoryArchiveError::DuplicateRecordId { id } if id == "memory"
    ));
}

#[test]
fn memory_archive_import_preserves_scope_and_sensitivity() {
    let scope = MemoryScope::Project {
        workspace_id: workspace_id("00000000-0000-0000-0000-000000000402"),
    };
    let archive = export_memory_archive(
        &[record(
            "project-memory",
            scope.clone(),
            "project-specific fact",
            MemorySensitivity::Sensitive,
        )],
        MemoryRetentionPolicy::default(),
    )
    .expect("export archive");

    let outcome = import_memory_archive(&archive).expect("import archive");

    assert_eq!(
        outcome.report,
        MemoryImportReport {
            accepted: 1,
            skipped: 0,
            rejected: 0,
        }
    );
    assert_eq!(outcome.records[0].scope, scope);
    assert_eq!(outcome.records[0].sensitivity, MemorySensitivity::Sensitive);
}

#[test]
fn memory_archive_round_trip_returns_expected_records() {
    let scope = MemoryScope::Conversation(ConversationId::new());
    let records = vec![
        record(
            "memory-b",
            scope.clone(),
            "second fact",
            MemorySensitivity::Internal,
        ),
        record("memory-a", scope, "first fact", MemorySensitivity::Public),
    ];
    let archive =
        export_memory_archive(&records, MemoryRetentionPolicy::default()).expect("export archive");

    let outcome = import_memory_archive(&archive).expect("import archive");
    let ids = outcome
        .records
        .iter()
        .map(|record| record.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["memory-a", "memory-b"]);
    assert_eq!(outcome.records.len(), 2);
}
