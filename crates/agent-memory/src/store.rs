use std::collections::BTreeMap;

use crate::{MemoryError, MemoryRecord, MemoryScope};

pub trait MemoryStore {
    fn add(&mut self, record: MemoryRecord) -> Result<(), MemoryError>;

    fn list(&self, scope: &MemoryScope) -> Result<Vec<MemoryRecord>, MemoryError>;

    fn delete(&mut self, scope: &MemoryScope, id: &str) -> Result<bool, MemoryError>;
}

#[derive(Default)]
pub struct InMemoryMemoryStore {
    records: BTreeMap<MemoryScope, BTreeMap<String, MemoryRecord>>,
}

impl MemoryStore for InMemoryMemoryStore {
    fn add(&mut self, record: MemoryRecord) -> Result<(), MemoryError> {
        validate_record(&record)?;
        let scope = record.scope.clone();
        let records = self.records.entry(scope).or_default();

        if records.contains_key(&record.id) {
            return Err(MemoryError::DuplicateRecord(record.id));
        }

        records.insert(record.id.clone(), record);
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
        Ok(records.remove(id).is_some())
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
