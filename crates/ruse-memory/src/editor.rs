use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    MemoryArchive, MemoryArchiveError, MemoryError, MemoryImportOutcome, MemoryImportReport,
    MemoryRecord, MemoryRetentionPolicy, MemoryScope, MemorySensitivity, MemoryTombstone,
    export_memory_archive, import_memory_archive,
};

const PREVIEW_LIMIT_BYTES: usize = 120;

pub trait MemoryEditor {
    fn add_record(&mut self, record: MemoryRecord) -> Result<(), MemoryError>;

    fn active_records(&self) -> Vec<MemoryRecord>;

    fn summaries(&self) -> Vec<MemoryEditorSummary>;

    fn list_summaries(&self, scope: &MemoryScope) -> Result<Vec<MemoryEditorSummary>, MemoryError>;

    fn inspect_summary(&self, scope: &MemoryScope, id: &str) -> Option<MemoryEditorSummary>;

    fn delete_memory(
        &mut self,
        scope: &MemoryScope,
        id: &str,
        reason: &str,
        source_event_id: Option<ruse_types::EventId>,
    ) -> Result<MemoryTombstone, MemoryError>;

    fn tombstones(&self) -> Vec<MemoryTombstone>;

    fn upsert_embedding_marker(&mut self, record_id: &str) -> Result<(), MemoryError>;

    fn embedding_marker_count(&self) -> usize;

    fn rebuild_index(&mut self) -> Result<MemoryIndexRebuildReport, MemoryError>;

    fn export_archive(
        &self,
        policy: MemoryRetentionPolicy,
    ) -> Result<MemoryArchive, MemoryArchiveError>;

    fn import_archive(
        &mut self,
        archive: &MemoryArchive,
        policy: MemoryImportPolicy,
    ) -> Result<MemoryImportOutcome, MemoryArchiveError>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryEditorSummary {
    pub id: String,
    pub scope: MemoryScope,
    pub sensitivity: MemorySensitivity,
    pub confidence: u8,
    pub source_description: String,
    pub text_preview: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryIndexRebuildReport {
    pub indexed_records: usize,
    pub skipped_tombstoned: usize,
    pub stale_embeddings_removed: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MemoryImportPolicy {
    pub tombstone_conflicts: MemoryTombstoneConflictPolicy,
}

impl Default for MemoryImportPolicy {
    fn default() -> Self {
        Self {
            tombstone_conflicts: MemoryTombstoneConflictPolicy::Reject,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MemoryTombstoneConflictPolicy {
    Reject,
    Skip,
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryMemoryEditor {
    records: BTreeMap<MemoryKey, MemoryRecord>,
    tombstones: BTreeMap<MemoryKey, MemoryTombstone>,
    embedding_record_ids: BTreeSet<String>,
}

impl MemoryEditor for InMemoryMemoryEditor {
    fn add_record(&mut self, record: MemoryRecord) -> Result<(), MemoryError> {
        validate_record(&record)?;
        let key = MemoryKey::from_record(&record);
        if self.tombstones.contains_key(&key) {
            return Err(MemoryError::InvalidRecord(format!(
                "memory record is tombstoned: {}",
                redact_secret_like_text(&record.id)
            )));
        }
        if self.records.contains_key(&key) {
            return Err(MemoryError::DuplicateRecord(record.id));
        }
        self.records.insert(key, record);
        Ok(())
    }

    fn active_records(&self) -> Vec<MemoryRecord> {
        self.records.values().cloned().collect()
    }

    fn summaries(&self) -> Vec<MemoryEditorSummary> {
        self.records.values().map(memory_summary).collect()
    }

    fn list_summaries(&self, scope: &MemoryScope) -> Result<Vec<MemoryEditorSummary>, MemoryError> {
        Ok(self
            .records
            .values()
            .filter(|record| &record.scope == scope)
            .map(memory_summary)
            .collect())
    }

    fn inspect_summary(&self, scope: &MemoryScope, id: &str) -> Option<MemoryEditorSummary> {
        self.records
            .get(&MemoryKey::new(scope.clone(), id))
            .map(memory_summary)
    }

    fn delete_memory(
        &mut self,
        scope: &MemoryScope,
        id: &str,
        reason: &str,
        source_event_id: Option<ruse_types::EventId>,
    ) -> Result<MemoryTombstone, MemoryError> {
        let key = MemoryKey::new(scope.clone(), id);
        if let Some(tombstone) = self.tombstones.get(&key) {
            return Ok(tombstone.clone());
        }
        if !self.records.contains_key(&key) {
            return Err(MemoryError::InvalidRecord(format!(
                "memory record not found: {}",
                redact_secret_like_text(id)
            )));
        }
        self.records.remove(&key);
        let tombstone =
            MemoryTombstone::new(scope.clone(), id.to_string(), reason, source_event_id);
        self.tombstones.insert(key, tombstone.clone());
        Ok(tombstone)
    }

    fn tombstones(&self) -> Vec<MemoryTombstone> {
        self.tombstones.values().cloned().collect()
    }

    fn upsert_embedding_marker(&mut self, record_id: &str) -> Result<(), MemoryError> {
        let record_id = record_id.trim();
        if record_id.is_empty() {
            return Err(MemoryError::InvalidEmbedding(
                "embedding record id cannot be empty".to_string(),
            ));
        }
        self.embedding_record_ids.insert(record_id.to_string());
        Ok(())
    }

    fn embedding_marker_count(&self) -> usize {
        self.embedding_record_ids.len()
    }

    fn rebuild_index(&mut self) -> Result<MemoryIndexRebuildReport, MemoryError> {
        let active_ids = self
            .records
            .values()
            .map(|record| record.id.clone())
            .collect::<BTreeSet<_>>();
        let before = self.embedding_record_ids.len();
        self.embedding_record_ids
            .retain(|record_id| active_ids.contains(record_id));
        let stale_embeddings_removed = before - self.embedding_record_ids.len();

        Ok(MemoryIndexRebuildReport {
            indexed_records: self.records.len(),
            skipped_tombstoned: self.tombstones.len(),
            stale_embeddings_removed,
        })
    }

    fn export_archive(
        &self,
        policy: MemoryRetentionPolicy,
    ) -> Result<MemoryArchive, MemoryArchiveError> {
        let mut archive = export_memory_archive(&self.active_records(), policy)?;
        archive.tombstones = self.tombstones();
        Ok(archive)
    }

    fn import_archive(
        &mut self,
        archive: &MemoryArchive,
        policy: MemoryImportPolicy,
    ) -> Result<MemoryImportOutcome, MemoryArchiveError> {
        let imported = import_memory_archive(archive)?;
        validate_tombstones(&archive.tombstones)?;

        let archive_tombstone_keys = archive
            .tombstones
            .iter()
            .map(MemoryKey::from_tombstone)
            .collect::<BTreeSet<_>>();
        let mut accepted = Vec::new();
        let mut skipped = 0_usize;

        for record in imported.records {
            let key = MemoryKey::from_record(&record);
            let tombstone_conflict =
                self.tombstones.contains_key(&key) || archive_tombstone_keys.contains(&key);
            if tombstone_conflict {
                match policy.tombstone_conflicts {
                    MemoryTombstoneConflictPolicy::Reject => {
                        return Err(MemoryArchiveError::TombstoneConflict {
                            id: record.id.clone(),
                        });
                    }
                    MemoryTombstoneConflictPolicy::Skip => {
                        skipped += 1;
                        continue;
                    }
                }
            }
            if self.records.contains_key(&key) {
                skipped += 1;
                continue;
            }
            accepted.push(record);
        }

        for tombstone in &archive.tombstones {
            let key = MemoryKey::from_tombstone(tombstone);
            self.records.remove(&key);
            self.tombstones.insert(key, tombstone.clone());
        }
        for record in &accepted {
            self.records
                .insert(MemoryKey::from_record(record), record.clone());
        }

        Ok(MemoryImportOutcome {
            report: MemoryImportReport {
                accepted: accepted.len(),
                skipped,
                rejected: 0,
            },
            records: accepted,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MemoryKey {
    scope: MemoryScope,
    id: String,
}

impl MemoryKey {
    fn new(scope: MemoryScope, id: &str) -> Self {
        Self {
            scope,
            id: id.to_string(),
        }
    }

    fn from_record(record: &MemoryRecord) -> Self {
        Self::new(record.scope.clone(), &record.id)
    }

    fn from_tombstone(tombstone: &MemoryTombstone) -> Self {
        Self::new(tombstone.scope.clone(), &tombstone.record_id)
    }
}

fn memory_summary(record: &MemoryRecord) -> MemoryEditorSummary {
    MemoryEditorSummary {
        id: record.id.clone(),
        scope: record.scope.clone(),
        sensitivity: record.sensitivity,
        confidence: record.confidence,
        source_description: redact_secret_like_text(&record.source.description),
        text_preview: safe_preview(&record.text, record.sensitivity),
    }
}

fn safe_preview(value: &str, sensitivity: MemorySensitivity) -> Option<String> {
    match sensitivity {
        MemorySensitivity::Sensitive => None,
        MemorySensitivity::Public | MemorySensitivity::Internal => Some(truncate_utf8(
            &redact_secret_like_text(value),
            PREVIEW_LIMIT_BYTES,
        )),
    }
}

fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }

    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &value[..end])
}

fn validate_record(record: &MemoryRecord) -> Result<(), MemoryError> {
    if record.id.trim().is_empty() {
        return Err(MemoryError::InvalidRecord(
            "memory id cannot be empty".to_string(),
        ));
    }
    if record.text.trim().is_empty() {
        return Err(MemoryError::InvalidRecord(
            "memory text cannot be empty".to_string(),
        ));
    }
    if record.source.description.trim().is_empty() {
        return Err(MemoryError::InvalidRecord(
            "memory source description cannot be empty".to_string(),
        ));
    }
    if !(1..=100).contains(&record.confidence) {
        return Err(MemoryError::InvalidRecord(
            "memory confidence must be between 1 and 100".to_string(),
        ));
    }
    Ok(())
}

fn validate_tombstones(tombstones: &[MemoryTombstone]) -> Result<(), MemoryArchiveError> {
    let mut seen = BTreeSet::new();
    for tombstone in tombstones {
        if tombstone.tombstone_id.trim().is_empty() {
            return Err(MemoryArchiveError::InvalidTombstone {
                id: tombstone.record_id.clone(),
                reason: "tombstone id cannot be empty".to_string(),
            });
        }
        if tombstone.record_id.trim().is_empty() {
            return Err(MemoryArchiveError::InvalidTombstone {
                id: tombstone.tombstone_id.clone(),
                reason: "record id cannot be empty".to_string(),
            });
        }
        if !seen.insert(MemoryKey::from_tombstone(tombstone)) {
            return Err(MemoryArchiveError::InvalidTombstone {
                id: tombstone.record_id.clone(),
                reason: "duplicate tombstone record id".to_string(),
            });
        }
    }
    Ok(())
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            if is_secret_like(token) {
                "[redacted]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
}
