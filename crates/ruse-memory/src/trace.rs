use serde::{Deserialize, Serialize};

use crate::{
    MemoryRejectedCandidate, MemoryRetrievalReason, MemoryScope, MemorySelectedCandidate,
    MemorySensitivity,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;
const PREVIEW_LIMIT_BYTES: usize = 120;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryInfluenceTrace {
    pub query_hash: String,
    pub selected_count: usize,
    pub rejected_count: usize,
    pub entries: Vec<MemoryInfluenceEntry>,
}

impl MemoryInfluenceTrace {
    #[must_use]
    pub fn empty(query_hash: String) -> Self {
        Self {
            query_hash,
            selected_count: 0,
            rejected_count: 0,
            entries: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_rerank(
        query_hash: String,
        selected: &[MemorySelectedCandidate],
        rejected: &[MemoryRejectedCandidate],
    ) -> Self {
        let mut entries = Vec::with_capacity(selected.len() + rejected.len());
        entries.extend(selected.iter().map(MemoryInfluenceEntry::from_selected));
        entries.extend(rejected.iter().map(MemoryInfluenceEntry::from_rejected));

        Self {
            query_hash,
            selected_count: selected.len(),
            rejected_count: rejected.len(),
            entries,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryInfluenceEntry {
    pub memory_id: String,
    pub scope: MemoryScope,
    pub sensitivity: MemorySensitivity,
    pub selected: bool,
    pub rank: usize,
    pub lexical_score: f32,
    pub semantic_score: Option<f32>,
    pub combined_score: f32,
    pub matched_terms: Vec<String>,
    pub reasons: Vec<MemoryRetrievalReason>,
    pub preview: Option<String>,
}

impl MemoryInfluenceEntry {
    fn from_selected(candidate: &MemorySelectedCandidate) -> Self {
        Self {
            memory_id: candidate.candidate.record.id.clone(),
            scope: candidate.candidate.record.scope.clone(),
            sensitivity: candidate.candidate.record.sensitivity,
            selected: true,
            rank: candidate.rank,
            lexical_score: candidate.candidate.lexical_score,
            semantic_score: candidate.candidate.semantic_score,
            combined_score: candidate.candidate.combined_score,
            matched_terms: candidate.candidate.matched_terms.clone(),
            reasons: candidate.reasons.clone(),
            preview: safe_preview(
                &candidate.candidate.record.text,
                candidate.candidate.record.sensitivity,
            ),
        }
    }

    fn from_rejected(candidate: &MemoryRejectedCandidate) -> Self {
        Self {
            memory_id: candidate.candidate.record.id.clone(),
            scope: candidate.candidate.record.scope.clone(),
            sensitivity: candidate.candidate.record.sensitivity,
            selected: false,
            rank: candidate.rank,
            lexical_score: candidate.candidate.lexical_score,
            semantic_score: candidate.candidate.semantic_score,
            combined_score: candidate.candidate.combined_score,
            matched_terms: candidate.candidate.matched_terms.clone(),
            reasons: candidate.reasons.clone(),
            preview: safe_preview(
                &candidate.candidate.record.text,
                candidate.candidate.record.sensitivity,
            ),
        }
    }
}

pub(crate) fn stable_trace_hash(value: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in value.trim().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("fnv1a64:{hash:016x}")
}

fn safe_preview(value: &str, sensitivity: MemorySensitivity) -> Option<String> {
    match sensitivity {
        MemorySensitivity::Sensitive => Some("[redacted]".to_string()),
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
