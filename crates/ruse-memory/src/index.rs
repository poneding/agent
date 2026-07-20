use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{EmbeddingVector, MemoryError, MemoryRecord, MemoryScope, MemorySensitivity};

pub trait MemoryIndex {
    fn upsert_record(&mut self, record: MemoryRecord) -> Result<(), MemoryError>;

    fn upsert_embedding(&mut self, embedding: EmbeddingVector) -> Result<(), MemoryError>;

    fn query(&self, query: HybridMemoryQuery)
    -> Result<Vec<MemoryRetrievalCandidate>, MemoryError>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HybridMemoryQuery {
    pub query_text: String,
    pub query_vector: Option<Vec<f32>>,
    pub allowed_scopes: Vec<MemoryScope>,
    pub sensitivity_ceiling: MemorySensitivity,
    pub lexical_weight: f32,
    pub semantic_weight: f32,
    pub top_k: usize,
    pub minimum_score: f32,
    pub prompt_budget_tokens: Option<u32>,
}

impl HybridMemoryQuery {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        query_text: impl Into<String>,
        query_vector: Option<Vec<f32>>,
        allowed_scopes: Vec<MemoryScope>,
        sensitivity_ceiling: MemorySensitivity,
        lexical_weight: f32,
        semantic_weight: f32,
        top_k: usize,
        minimum_score: f32,
        prompt_budget_tokens: Option<u32>,
    ) -> Result<Self, MemoryError> {
        let query_text = query_text.into();
        let query_text = query_text.trim();
        if query_text.is_empty() {
            return Err(invalid_query("memory query text cannot be empty"));
        }
        if allowed_scopes.is_empty() {
            return Err(invalid_query("memory query scope list cannot be empty"));
        }
        if top_k == 0 {
            return Err(invalid_query(
                "memory query top_k must be greater than zero",
            ));
        }
        validate_weight("lexical weight", lexical_weight)?;
        validate_weight("semantic weight", semantic_weight)?;
        if !minimum_score.is_finite() || minimum_score < 0.0 {
            return Err(invalid_query(
                "memory query minimum score must be finite and non-negative",
            ));
        }
        if let Some(vector) = &query_vector {
            if vector.is_empty() {
                return Err(invalid_query("memory query vector cannot be empty"));
            }
            if vector.iter().any(|value| !value.is_finite()) {
                return Err(invalid_query(
                    "memory query vector values must be finite numbers",
                ));
            }
        }

        Ok(Self {
            query_text: query_text.to_string(),
            query_vector,
            allowed_scopes,
            sensitivity_ceiling,
            lexical_weight,
            semantic_weight,
            top_k,
            minimum_score,
            prompt_budget_tokens,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryRetrievalCandidate {
    pub record: MemoryRecord,
    pub lexical_score: f32,
    pub semantic_score: Option<f32>,
    pub combined_score: f32,
    pub matched_terms: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryIndexStats {
    pub record_count: usize,
    pub embedding_count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryMemoryIndex {
    records: BTreeMap<MemoryRecordKey, MemoryRecord>,
    embeddings: BTreeMap<String, EmbeddingVector>,
}

impl InMemoryMemoryIndex {
    #[must_use]
    pub fn stats(&self) -> MemoryIndexStats {
        MemoryIndexStats {
            record_count: self.records.len(),
            embedding_count: self.embeddings.len(),
        }
    }

    fn matching_record_keys(&self, record_id: &str) -> Vec<MemoryRecordKey> {
        self.records
            .keys()
            .filter(|key| key.record_id == record_id)
            .cloned()
            .collect()
    }
}

impl MemoryIndex for InMemoryMemoryIndex {
    fn upsert_record(&mut self, record: MemoryRecord) -> Result<(), MemoryError> {
        validate_record(&record)?;
        self.records.insert(MemoryRecordKey::from(&record), record);
        Ok(())
    }

    fn upsert_embedding(&mut self, embedding: EmbeddingVector) -> Result<(), MemoryError> {
        let keys = self.matching_record_keys(&embedding.record_id);
        if keys.is_empty() {
            return Err(MemoryError::InvalidEmbedding(format!(
                "orphan embedding has no indexed memory record: {}",
                redact_secret_like_text(&embedding.record_id)
            )));
        }
        if keys.len() > 1 {
            return Err(MemoryError::InvalidEmbedding(format!(
                "embedding record id is ambiguous across memory scopes: {}",
                redact_secret_like_text(&embedding.record_id)
            )));
        }
        self.embeddings
            .insert(embedding.record_id.clone(), embedding);
        Ok(())
    }

    fn query(
        &self,
        query: HybridMemoryQuery,
    ) -> Result<Vec<MemoryRetrievalCandidate>, MemoryError> {
        let query_terms = normalized_terms(&query.query_text);
        if query_terms.is_empty() {
            return Err(invalid_query("memory query terms cannot be empty"));
        }
        let allowed_scopes = query.allowed_scopes.iter().collect::<BTreeSet<_>>();
        let mut candidates = Vec::new();

        for record in self.records.values() {
            if !allowed_scopes.contains(&record.scope)
                || !sensitivity_allowed(record.sensitivity, query.sensitivity_ceiling)
            {
                continue;
            }

            let matched_terms = matched_terms(&query_terms, &record.text);
            let lexical_score = if query_terms.is_empty() {
                0.0
            } else {
                matched_terms.len() as f32 / query_terms.len() as f32
            };
            let semantic_score = semantic_score(
                query.query_vector.as_deref(),
                self.embeddings
                    .get(&record.id)
                    .map(|embedding| embedding.values.as_slice()),
            )?;
            let combined_score = combined_score(
                lexical_score,
                semantic_score,
                query.lexical_weight,
                query.semantic_weight,
            );

            if !combined_score.is_finite() {
                return Err(invalid_query("memory query produced a non-finite score"));
            }
            if combined_score < query.minimum_score {
                continue;
            }
            if matched_terms.is_empty() && semantic_score.unwrap_or(0.0) <= 0.0 {
                continue;
            }

            candidates.push(MemoryRetrievalCandidate {
                record: record.clone(),
                lexical_score,
                semantic_score,
                combined_score,
                matched_terms,
            });
        }

        candidates.sort_by(compare_candidates);
        candidates.truncate(query.top_k);
        Ok(candidates)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct MemoryRecordKey {
    scope: MemoryScope,
    record_id: String,
}

impl From<&MemoryRecord> for MemoryRecordKey {
    fn from(record: &MemoryRecord) -> Self {
        Self {
            scope: record.scope.clone(),
            record_id: record.id.clone(),
        }
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

fn validate_weight(label: &str, value: f32) -> Result<(), MemoryError> {
    if !value.is_finite() || value < 0.0 {
        return Err(invalid_query(format!(
            "memory query {label} must be finite and non-negative"
        )));
    }
    Ok(())
}

fn normalized_terms(value: &str) -> Vec<String> {
    let mut terms = value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter_map(|term| {
            let term = term.trim();
            if term.is_empty() {
                None
            } else {
                Some(term.to_ascii_lowercase())
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    terms.sort();
    terms
}

fn matched_terms(query_terms: &[String], text: &str) -> Vec<String> {
    let text_terms = normalized_terms(text).into_iter().collect::<BTreeSet<_>>();
    query_terms
        .iter()
        .filter(|term| text_terms.contains(*term))
        .cloned()
        .collect()
}

fn semantic_score(
    query_vector: Option<&[f32]>,
    record_vector: Option<&[f32]>,
) -> Result<Option<f32>, MemoryError> {
    let (Some(query_vector), Some(record_vector)) = (query_vector, record_vector) else {
        return Ok(None);
    };
    if query_vector.len() != record_vector.len() {
        return Err(invalid_query(format!(
            "memory query vector dimension mismatch: expected {}, got {}",
            record_vector.len(),
            query_vector.len()
        )));
    }
    let cosine = cosine_similarity(query_vector, record_vector)?;
    Ok(Some(cosine.max(0.0)))
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> Result<f32, MemoryError> {
    let mut dot = 0.0_f32;
    let mut left_norm = 0.0_f32;
    let mut right_norm = 0.0_f32;
    for (left_value, right_value) in left.iter().zip(right) {
        dot += left_value * right_value;
        left_norm += left_value * left_value;
        right_norm += right_value * right_value;
    }
    if left_norm == 0.0 || right_norm == 0.0 {
        return Ok(0.0);
    }
    let score = dot / (left_norm.sqrt() * right_norm.sqrt());
    if !score.is_finite() {
        return Err(invalid_query("memory semantic score must be finite"));
    }
    Ok(score.clamp(-1.0, 1.0))
}

fn combined_score(
    lexical_score: f32,
    semantic_score: Option<f32>,
    lexical_weight: f32,
    semantic_weight: f32,
) -> f32 {
    (lexical_score * lexical_weight) + (semantic_score.unwrap_or(0.0) * semantic_weight)
}

fn compare_candidates(
    left: &MemoryRetrievalCandidate,
    right: &MemoryRetrievalCandidate,
) -> Ordering {
    right
        .combined_score
        .total_cmp(&left.combined_score)
        .then_with(|| right.lexical_score.total_cmp(&left.lexical_score))
        .then_with(|| left.record.id.cmp(&right.record.id))
}

fn sensitivity_allowed(record: MemorySensitivity, ceiling: MemorySensitivity) -> bool {
    sensitivity_rank(record) <= sensitivity_rank(ceiling)
}

const fn sensitivity_rank(value: MemorySensitivity) -> u8 {
    match value {
        MemorySensitivity::Public => 0,
        MemorySensitivity::Internal => 1,
        MemorySensitivity::Sensitive => 2,
    }
}

fn invalid_query(message: impl Into<String>) -> MemoryError {
    MemoryError::InvalidQuery(redact_secret_like_text(&message.into()))
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
