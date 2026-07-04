use serde::{Deserialize, Serialize};

use crate::{MemoryScope, MemoryStore};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryAuditAction {
    Added,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryAuditEntry {
    pub action: MemoryAuditAction,
    pub scope: MemoryScope,
    pub record_id: String,
    pub reason: String,
}

pub trait AuditedMemoryStore: MemoryStore {
    fn audit_entries(&self) -> Vec<MemoryAuditEntry>;
}
