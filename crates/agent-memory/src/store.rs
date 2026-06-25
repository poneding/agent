use std::collections::BTreeMap;

use crate::{
    AuditedMemoryStore, MemoryAuditAction, MemoryAuditEntry, MemoryError, MemoryRecord, MemoryScope,
};

pub trait MemoryStore {
    fn add(&mut self, record: MemoryRecord) -> Result<(), MemoryError>;

    fn list(&self, scope: &MemoryScope) -> Result<Vec<MemoryRecord>, MemoryError>;

    fn delete(&mut self, scope: &MemoryScope, id: &str) -> Result<bool, MemoryError>;
}

#[derive(Default)]
pub struct InMemoryMemoryStore {
    records: BTreeMap<MemoryScope, BTreeMap<String, MemoryRecord>>,
    audit: Vec<MemoryAuditEntry>,
}

impl MemoryStore for InMemoryMemoryStore {
    fn add(&mut self, record: MemoryRecord) -> Result<(), MemoryError> {
        validate_record(&record)?;
        let scope = record.scope.clone();
        let records = self.records.entry(scope.clone()).or_default();

        if records.contains_key(&record.id) {
            return Err(MemoryError::DuplicateRecord(record.id));
        }

        let record_id = record.id.clone();
        records.insert(record_id.clone(), record);
        self.audit.push(MemoryAuditEntry {
            action: MemoryAuditAction::Added,
            scope,
            record_id,
            reason: "memory record added".to_string(),
        });
        Ok(())
    }

    fn list(&self, scope: &MemoryScope) -> Result<Vec<MemoryRecord>, MemoryError> {
        Ok(self
            .records
            .get(scope)
            .map(|records| records.values().cloned().collect())
            .unwrap_or_default())
    }

    fn delete(&mut self, scope: &MemoryScope, id: &str) -> Result<bool, MemoryError> {
        let Some(records) = self.records.get_mut(scope) else {
            return Ok(false);
        };
        let removed = records.remove(id).is_some();
        if removed {
            self.audit.push(MemoryAuditEntry {
                action: MemoryAuditAction::Deleted,
                scope: scope.clone(),
                record_id: id.to_string(),
                reason: "memory record deleted".to_string(),
            });
        }
        Ok(removed)
    }
}

impl AuditedMemoryStore for InMemoryMemoryStore {
    fn audit_entries(&self) -> Vec<MemoryAuditEntry> {
        self.audit.clone()
    }
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
