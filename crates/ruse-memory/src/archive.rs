use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{MemoryRecord, MemoryScope, MemorySensitivity, MemorySource, MemoryTombstone};

pub const MEMORY_ARCHIVE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryArchive {
    pub schema_version: u32,
    pub manifest: MemoryArchiveManifest,
    pub records: Vec<MemoryArchiveRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tombstones: Vec<MemoryTombstone>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryArchiveManifest {
    pub exported_by: String,
    pub record_count: usize,
    pub sensitive_record_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryArchiveRecord {
    pub id: String,
    pub scope: MemoryScope,
    pub text: String,
    pub source: MemorySource,
    pub confidence: u8,
    pub sensitivity: MemorySensitivity,
}

impl From<MemoryRecord> for MemoryArchiveRecord {
    fn from(record: MemoryRecord) -> Self {
        Self {
            id: record.id,
            scope: record.scope,
            text: record.text,
            source: record.source,
            confidence: record.confidence,
            sensitivity: record.sensitivity,
        }
    }
}

impl MemoryArchiveRecord {
    fn to_memory_record(&self) -> MemoryRecord {
        MemoryRecord {
            id: self.id.clone(),
            scope: self.scope.clone(),
            text: self.text.clone(),
            source: self.source.clone(),
            confidence: self.confidence,
            sensitivity: self.sensitivity,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRetentionPolicy {
    pub include_sensitive: bool,
    pub max_records: Option<usize>,
}

impl Default for MemoryRetentionPolicy {
    fn default() -> Self {
        Self {
            include_sensitive: true,
            max_records: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryImportOutcome {
    pub records: Vec<MemoryRecord>,
    pub report: MemoryImportReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryImportReport {
    pub accepted: usize,
    pub skipped: usize,
    pub rejected: usize,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MemoryArchiveError {
    #[error(
        "unsupported memory archive schema version {version}; supported version is {supported}"
    )]
    UnsupportedSchemaVersion { version: u32, supported: u32 },
    #[error("memory archive manifest does not match records")]
    ManifestMismatch,
    #[error("duplicate memory archive record id: {id}")]
    DuplicateRecordId { id: String },
    #[error("memory archive record {id} is invalid: {reason}")]
    InvalidRecord { id: String, reason: String },
    #[error("memory archive tombstone {id} is invalid: {reason}")]
    InvalidTombstone { id: String, reason: String },
    #[error("memory archive record conflicts with tombstone: {id}")]
    TombstoneConflict { id: String },
}

pub fn export_memory_archive(
    records: &[MemoryRecord],
    policy: MemoryRetentionPolicy,
) -> Result<MemoryArchive, MemoryArchiveError> {
    let mut records = records
        .iter()
        .filter(|record| {
            policy.include_sensitive || record.sensitivity != MemorySensitivity::Sensitive
        })
        .cloned()
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.id.cmp(&right.id));

    if let Some(max_records) = policy.max_records {
        records.truncate(max_records);
    }

    let records = records
        .into_iter()
        .map(MemoryArchiveRecord::from)
        .collect::<Vec<_>>();
    let sensitive_record_count = records
        .iter()
        .filter(|record| record.sensitivity == MemorySensitivity::Sensitive)
        .count();

    Ok(MemoryArchive {
        schema_version: MEMORY_ARCHIVE_SCHEMA_VERSION,
        manifest: MemoryArchiveManifest {
            exported_by: "ruse-memory".to_string(),
            record_count: records.len(),
            sensitive_record_count,
        },
        records,
        tombstones: Vec::new(),
    })
}

pub fn import_memory_archive(
    archive: &MemoryArchive,
) -> Result<MemoryImportOutcome, MemoryArchiveError> {
    if archive.schema_version != MEMORY_ARCHIVE_SCHEMA_VERSION {
        return Err(MemoryArchiveError::UnsupportedSchemaVersion {
            version: archive.schema_version,
            supported: MEMORY_ARCHIVE_SCHEMA_VERSION,
        });
    }
    validate_manifest(archive)?;

    let mut seen_ids = BTreeSet::new();
    let mut records = Vec::with_capacity(archive.records.len());
    for archive_record in &archive.records {
        if !seen_ids.insert(archive_record.id.clone()) {
            return Err(MemoryArchiveError::DuplicateRecordId {
                id: archive_record.id.clone(),
            });
        }
        let record = archive_record.to_memory_record();
        validate_record(&record)?;
        records.push(record);
    }

    Ok(MemoryImportOutcome {
        report: MemoryImportReport {
            accepted: records.len(),
            skipped: 0,
            rejected: 0,
        },
        records,
    })
}

fn validate_manifest(archive: &MemoryArchive) -> Result<(), MemoryArchiveError> {
    let sensitive_record_count = archive
        .records
        .iter()
        .filter(|record| record.sensitivity == MemorySensitivity::Sensitive)
        .count();
    if archive.manifest.record_count != archive.records.len()
        || archive.manifest.sensitive_record_count != sensitive_record_count
    {
        return Err(MemoryArchiveError::ManifestMismatch);
    }
    Ok(())
}

fn validate_record(record: &MemoryRecord) -> Result<(), MemoryArchiveError> {
    let invalid = |reason: &str| MemoryArchiveError::InvalidRecord {
        id: record.id.clone(),
        reason: reason.to_string(),
    };

    if record.id.trim().is_empty() {
        return Err(invalid("memory id cannot be empty"));
    }
    if record.text.trim().is_empty() {
        return Err(invalid("memory text cannot be empty"));
    }
    if record.source.description.trim().is_empty() {
        return Err(invalid("memory source description cannot be empty"));
    }
    if !(1..=100).contains(&record.confidence) {
        return Err(invalid("memory confidence must be between 1 and 100"));
    }
    Ok(())
}
