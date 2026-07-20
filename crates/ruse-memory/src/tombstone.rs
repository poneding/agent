use ruse_types::EventId;
use serde::{Deserialize, Serialize};

use crate::MemoryScope;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryTombstone {
    pub tombstone_id: String,
    pub scope: MemoryScope,
    pub record_id: String,
    pub reason: String,
    pub source_event_id: Option<EventId>,
}

impl MemoryTombstone {
    pub fn new(
        scope: MemoryScope,
        record_id: impl Into<String>,
        reason: impl Into<String>,
        source_event_id: Option<EventId>,
    ) -> Self {
        let record_id = record_id.into().trim().to_string();
        let reason = redact_secret_like_text(reason.into().trim());
        let tombstone_id = tombstone_id(&scope, &record_id);
        Self {
            tombstone_id,
            scope,
            record_id,
            reason,
            source_event_id,
        }
    }
}

fn tombstone_id(scope: &MemoryScope, record_id: &str) -> String {
    let input = serde_json::json!({
        "scope": scope,
        "record_id": record_id,
    });
    let json = serde_json::to_string(&input).expect("tombstone hash input serializes");
    let mut hash = FNV_OFFSET_BASIS;
    for byte in json.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("tombstone:fnv1a64:{hash:016x}")
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
