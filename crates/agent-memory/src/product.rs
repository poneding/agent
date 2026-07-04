use std::collections::BTreeSet;

use agent_types::EventId;
use serde::{Deserialize, Serialize};

use crate::{
    MemoryInfluenceEntry, MemoryInfluenceTrace, MemoryProposalSummary, MemoryRecord,
    MemoryRejectedCandidate, MemoryRetrievalReason, MemoryRetrievalResult, MemoryScope,
    MemorySelectedCandidate, MemorySensitivity, MemoryTombstone, MemoryWriteProposal,
};

const TEXT_PREVIEW_LIMIT_BYTES: usize = 120;
const LABEL_LIMIT_BYTES: usize = 120;
const DIAGNOSTIC_LIMIT_BYTES: usize = 160;
const MAX_RECORD_SUMMARIES: usize = 100;
const MAX_PROPOSAL_SUMMARIES: usize = 100;
const MAX_RETRIEVAL_CANDIDATES: usize = 50;
const MAX_INFLUENCE_ENTRIES: usize = 50;
const MAX_MATCHED_TERMS: usize = 12;
const MAX_DIAGNOSTICS: usize = 25;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryProductSnapshot {
    pub scope_label: String,
    pub records: Vec<MemoryRecordSummary>,
    pub records_truncated: bool,
    pub proposals: Vec<MemoryProposalSummary>,
    pub proposals_truncated: bool,
    pub retrieval: Option<MemoryRetrievalPreview>,
    pub influence: Option<MemoryInfluenceSummary>,
    pub retention: MemoryRetentionControlSummary,
    pub tombstone_count: u32,
    pub diagnostics: Vec<MemoryPrivacyDiagnostic>,
    pub diagnostics_truncated: bool,
}

impl MemoryProductSnapshot {
    #[must_use]
    pub fn empty(scope_label: impl AsRef<str>) -> Self {
        Self::from_parts(
            scope_label,
            &[],
            &[],
            None,
            None,
            MemoryRetentionControlSummary::default(),
            &[],
            Vec::new(),
        )
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        scope_label: impl AsRef<str>,
        records: &[MemoryRecord],
        proposals: &[MemoryWriteProposal],
        retrieval: Option<&MemoryRetrievalResult>,
        influence: Option<&MemoryInfluenceTrace>,
        retention: MemoryRetentionControlSummary,
        tombstones: &[MemoryTombstone],
        diagnostics: Vec<MemoryPrivacyDiagnostic>,
    ) -> Self {
        let tombstone_keys = tombstones
            .iter()
            .map(|tombstone| (tombstone.scope.clone(), tombstone.record_id.clone()))
            .collect::<BTreeSet<_>>();

        let mut records = records
            .iter()
            .filter(|record| !tombstone_keys.contains(&(record.scope.clone(), record.id.clone())))
            .map(MemoryRecordSummary::from_record)
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.scope
                .cmp(&right.scope)
                .then_with(|| left.id.cmp(&right.id))
        });
        let records_truncated = records.len() > MAX_RECORD_SUMMARIES;
        records.truncate(MAX_RECORD_SUMMARIES);

        let mut proposals = proposals
            .iter()
            .map(MemoryWriteProposal::safe_summary)
            .collect::<Vec<_>>();
        proposals.sort_by(|left, right| left.proposal_id.cmp(&right.proposal_id));
        let proposals_truncated = proposals.len() > MAX_PROPOSAL_SUMMARIES;
        proposals.truncate(MAX_PROPOSAL_SUMMARIES);

        let mut diagnostics = diagnostics
            .into_iter()
            .map(MemoryPrivacyDiagnostic::sanitize)
            .collect::<Vec<_>>();
        diagnostics.sort_by(|left, right| left.code.cmp(&right.code));
        let diagnostics_truncated = diagnostics.len() > MAX_DIAGNOSTICS;
        diagnostics.truncate(MAX_DIAGNOSTICS);

        Self {
            scope_label: safe_label(scope_label.as_ref()),
            records,
            records_truncated,
            proposals,
            proposals_truncated,
            retrieval: retrieval.map(MemoryRetrievalPreview::from_result),
            influence: influence.map(MemoryInfluenceSummary::from_trace),
            retention,
            tombstone_count: saturating_u32(tombstones.len()),
            diagnostics,
            diagnostics_truncated,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecordSummary {
    pub id: String,
    pub scope: MemoryScope,
    pub sensitivity: MemorySensitivity,
    pub confidence: u8,
    pub source_event_id: Option<EventId>,
    pub source_description: String,
    pub text_preview: Option<String>,
    pub preview_redacted: bool,
    pub preview_truncated: bool,
}

impl MemoryRecordSummary {
    #[must_use]
    pub fn from_record(record: &MemoryRecord) -> Self {
        let (text_preview, preview_redacted, preview_truncated) =
            safe_preview(&record.text, record.sensitivity);
        Self {
            id: safe_label(&record.id),
            scope: record.scope.clone(),
            sensitivity: record.sensitivity,
            confidence: record.confidence,
            source_event_id: record.source.event_id,
            source_description: safe_text(
                &record.source.description,
                LABEL_LIMIT_BYTES,
                MemorySensitivity::Internal,
            )
            .0
            .unwrap_or_default(),
            text_preview,
            preview_redacted,
            preview_truncated,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryRetrievalPreview {
    pub query_hash: String,
    pub selected_count: usize,
    pub rejected_count: usize,
    pub candidates: Vec<MemoryRetrievalCandidateSummary>,
    pub candidates_truncated: bool,
}

impl MemoryRetrievalPreview {
    #[must_use]
    pub fn from_result(result: &MemoryRetrievalResult) -> Self {
        let mut candidates = result
            .selected
            .iter()
            .map(MemoryRetrievalCandidateSummary::from_selected)
            .chain(
                result
                    .rejected
                    .iter()
                    .map(MemoryRetrievalCandidateSummary::from_rejected),
            )
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            left.rank
                .cmp(&right.rank)
                .then_with(|| right.selected.cmp(&left.selected))
                .then_with(|| left.memory_id.cmp(&right.memory_id))
        });
        let candidates_truncated = candidates.len() > MAX_RETRIEVAL_CANDIDATES;
        candidates.truncate(MAX_RETRIEVAL_CANDIDATES);

        Self {
            query_hash: safe_label(&result.trace.query_hash),
            selected_count: result.selected.len(),
            rejected_count: result.rejected.len(),
            candidates,
            candidates_truncated,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryRetrievalCandidateSummary {
    pub memory_id: String,
    pub scope: MemoryScope,
    pub sensitivity: MemorySensitivity,
    pub selected: bool,
    pub rank: usize,
    pub token_estimate: u32,
    pub lexical_score: f32,
    pub semantic_score: Option<f32>,
    pub combined_score: f32,
    pub matched_terms: Vec<String>,
    pub reasons: Vec<MemoryRetrievalReason>,
    pub preview: Option<String>,
    pub preview_redacted: bool,
    pub preview_truncated: bool,
}

impl MemoryRetrievalCandidateSummary {
    fn from_selected(candidate: &MemorySelectedCandidate) -> Self {
        let record = &candidate.candidate.record;
        let (preview, preview_redacted, preview_truncated) =
            safe_preview(&record.text, record.sensitivity);
        Self {
            memory_id: safe_label(&record.id),
            scope: record.scope.clone(),
            sensitivity: record.sensitivity,
            selected: true,
            rank: candidate.rank,
            token_estimate: candidate.token_estimate,
            lexical_score: candidate.candidate.lexical_score,
            semantic_score: candidate.candidate.semantic_score,
            combined_score: candidate.candidate.combined_score,
            matched_terms: safe_terms(&candidate.candidate.matched_terms),
            reasons: candidate.reasons.clone(),
            preview,
            preview_redacted,
            preview_truncated,
        }
    }

    fn from_rejected(candidate: &MemoryRejectedCandidate) -> Self {
        let record = &candidate.candidate.record;
        let (preview, preview_redacted, preview_truncated) =
            safe_preview(&record.text, record.sensitivity);
        Self {
            memory_id: safe_label(&record.id),
            scope: record.scope.clone(),
            sensitivity: record.sensitivity,
            selected: false,
            rank: candidate.rank,
            token_estimate: candidate.token_estimate,
            lexical_score: candidate.candidate.lexical_score,
            semantic_score: candidate.candidate.semantic_score,
            combined_score: candidate.candidate.combined_score,
            matched_terms: safe_terms(&candidate.candidate.matched_terms),
            reasons: candidate.reasons.clone(),
            preview,
            preview_redacted,
            preview_truncated,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryInfluenceSummary {
    pub query_hash: String,
    pub selected_count: usize,
    pub rejected_count: usize,
    pub entries: Vec<MemoryInfluenceEntrySummary>,
    pub entries_truncated: bool,
}

impl MemoryInfluenceSummary {
    #[must_use]
    pub fn from_trace(trace: &MemoryInfluenceTrace) -> Self {
        let mut entries = trace
            .entries
            .iter()
            .map(MemoryInfluenceEntrySummary::from_entry)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.rank
                .cmp(&right.rank)
                .then_with(|| right.selected.cmp(&left.selected))
                .then_with(|| left.memory_id.cmp(&right.memory_id))
        });
        let entries_truncated = entries.len() > MAX_INFLUENCE_ENTRIES;
        entries.truncate(MAX_INFLUENCE_ENTRIES);

        Self {
            query_hash: safe_label(&trace.query_hash),
            selected_count: trace.selected_count,
            rejected_count: trace.rejected_count,
            entries,
            entries_truncated,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryInfluenceEntrySummary {
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
    pub preview_redacted: bool,
    pub preview_truncated: bool,
}

impl MemoryInfluenceEntrySummary {
    fn from_entry(entry: &MemoryInfluenceEntry) -> Self {
        let (preview, preview_redacted, preview_truncated) = match entry.preview.as_deref() {
            Some("[redacted]") => (Some("[redacted]".to_string()), true, false),
            Some(preview) => safe_text(preview, TEXT_PREVIEW_LIMIT_BYTES, entry.sensitivity),
            None => (
                None,
                entry.sensitivity == MemorySensitivity::Sensitive,
                false,
            ),
        };

        Self {
            memory_id: safe_label(&entry.memory_id),
            scope: entry.scope.clone(),
            sensitivity: entry.sensitivity,
            selected: entry.selected,
            rank: entry.rank,
            lexical_score: entry.lexical_score,
            semantic_score: entry.semantic_score,
            combined_score: entry.combined_score,
            matched_terms: safe_terms(&entry.matched_terms),
            reasons: entry.reasons.clone(),
            preview,
            preview_redacted,
            preview_truncated,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryRetentionControlSummary {
    pub memory_enabled: bool,
    pub export_includes_sensitive: bool,
    pub export_max_records: Option<usize>,
    pub tombstones_retained: bool,
    pub sensitive_memory_requires_review: bool,
}

impl Default for MemoryRetentionControlSummary {
    fn default() -> Self {
        Self {
            memory_enabled: true,
            export_includes_sensitive: false,
            export_max_records: None,
            tombstones_retained: true,
            sensitive_memory_requires_review: true,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryPrivacyDiagnosticSeverity {
    Info,
    Warning,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryPrivacyDiagnostic {
    pub code: String,
    pub severity: MemoryPrivacyDiagnosticSeverity,
    pub message: String,
}

impl MemoryPrivacyDiagnostic {
    #[must_use]
    pub fn new(
        code: impl AsRef<str>,
        severity: MemoryPrivacyDiagnosticSeverity,
        message: impl AsRef<str>,
    ) -> Self {
        Self {
            code: safe_label(code.as_ref()),
            severity,
            message: safe_text(
                message.as_ref(),
                DIAGNOSTIC_LIMIT_BYTES,
                MemorySensitivity::Internal,
            )
            .0
            .unwrap_or_default(),
        }
    }

    fn sanitize(self) -> Self {
        Self::new(self.code, self.severity, self.message)
    }
}

fn safe_label(value: &str) -> String {
    safe_text(value, LABEL_LIMIT_BYTES, MemorySensitivity::Internal)
        .0
        .unwrap_or_default()
}

fn safe_preview(value: &str, sensitivity: MemorySensitivity) -> (Option<String>, bool, bool) {
    safe_text(value, TEXT_PREVIEW_LIMIT_BYTES, sensitivity)
}

fn safe_text(
    value: &str,
    max_bytes: usize,
    sensitivity: MemorySensitivity,
) -> (Option<String>, bool, bool) {
    if sensitivity == MemorySensitivity::Sensitive {
        return (Some("[redacted]".to_string()), true, false);
    }

    let redacted = redact_secret_like_text(value);
    let truncated = redacted.len() > max_bytes;
    (Some(truncate_utf8(&redacted, max_bytes)), false, truncated)
}

fn safe_terms(values: &[String]) -> Vec<String> {
    values
        .iter()
        .take(MAX_MATCHED_TERMS)
        .map(|value| safe_label(value))
        .collect()
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

fn saturating_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
