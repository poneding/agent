use std::cmp::Ordering;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    MemoryError, MemoryInfluenceTrace, MemoryRecord, MemoryRetrievalCandidate, stable_trace_hash,
};

pub trait MemoryReranker {
    fn rerank(&self, request: MemoryRerankRequest) -> Result<MemoryRetrievalResult, MemoryError>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryRerankRequest {
    pub query_text: String,
    pub candidates: Vec<MemoryRetrievalCandidate>,
    pub max_selected: usize,
    pub prompt_budget_tokens: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryRetrievalResult {
    pub selected: Vec<MemorySelectedCandidate>,
    pub rejected: Vec<MemoryRejectedCandidate>,
    pub trace: MemoryInfluenceTrace,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemorySelectedCandidate {
    pub candidate: MemoryRetrievalCandidate,
    pub rank: usize,
    pub token_estimate: u32,
    pub reasons: Vec<MemoryRetrievalReason>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemoryRejectedCandidate {
    pub candidate: MemoryRetrievalCandidate,
    pub rank: usize,
    pub token_estimate: u32,
    pub reasons: Vec<MemoryRetrievalReason>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRetrievalReason {
    Score,
    Scope,
    Sensitivity,
    Budget,
    Duplicate,
    BelowThreshold,
}

#[derive(Clone, Debug, Default)]
pub struct DeterministicMemoryReranker {
    _private: (),
}

impl MemoryReranker for DeterministicMemoryReranker {
    fn rerank(&self, request: MemoryRerankRequest) -> Result<MemoryRetrievalResult, MemoryError> {
        validate_request(&request)?;

        let query_hash = stable_trace_hash(&request.query_text);
        if request.candidates.is_empty() {
            return Ok(MemoryRetrievalResult {
                selected: Vec::new(),
                rejected: Vec::new(),
                trace: MemoryInfluenceTrace::empty(query_hash),
            });
        }

        let mut ranked = request
            .candidates
            .into_iter()
            .enumerate()
            .map(|(index, candidate)| RankedCandidate::new(index, candidate))
            .collect::<Result<Vec<_>, _>>()?;
        ranked.sort_by(compare_ranked_candidates);

        let mut seen_ids = BTreeSet::new();
        let mut selected = Vec::new();
        let mut rejected = Vec::new();
        let mut used_tokens = 0_u32;

        for (rank_index, ranked_candidate) in ranked.into_iter().enumerate() {
            let rank = rank_index + 1;
            let token_estimate = ranked_candidate.token_estimate;
            let candidate = ranked_candidate.candidate;

            if !seen_ids.insert(candidate.record.id.clone()) {
                rejected.push(reject(
                    candidate,
                    rank,
                    token_estimate,
                    MemoryRetrievalReason::Duplicate,
                ));
                continue;
            }

            if selected.len() >= request.max_selected {
                rejected.push(reject(
                    candidate,
                    rank,
                    token_estimate,
                    MemoryRetrievalReason::Score,
                ));
                continue;
            }

            if let Some(prompt_budget_tokens) = request.prompt_budget_tokens {
                let next_used = used_tokens.saturating_add(token_estimate);
                if next_used > prompt_budget_tokens {
                    rejected.push(reject(
                        candidate,
                        rank,
                        token_estimate,
                        MemoryRetrievalReason::Budget,
                    ));
                    continue;
                }
                used_tokens = next_used;
            }

            selected.push(MemorySelectedCandidate {
                candidate,
                rank,
                token_estimate,
                reasons: vec![MemoryRetrievalReason::Score],
            });
        }

        let trace = MemoryInfluenceTrace::from_rerank(query_hash, &selected, &rejected);
        Ok(MemoryRetrievalResult {
            selected,
            rejected,
            trace,
        })
    }
}

#[derive(Clone, Debug)]
struct RankedCandidate {
    original_index: usize,
    token_estimate: u32,
    candidate: MemoryRetrievalCandidate,
}

impl RankedCandidate {
    fn new(
        original_index: usize,
        candidate: MemoryRetrievalCandidate,
    ) -> Result<Self, MemoryError> {
        validate_candidate_scores(&candidate)?;
        let token_estimate = estimate_memory_tokens(&candidate.record);
        Ok(Self {
            original_index,
            token_estimate,
            candidate,
        })
    }
}

fn validate_request(request: &MemoryRerankRequest) -> Result<(), MemoryError> {
    if request.query_text.trim().is_empty() {
        return Err(invalid_rerank("query_text cannot be empty"));
    }
    if request.max_selected == 0 {
        return Err(invalid_rerank("max_selected must be greater than zero"));
    }
    Ok(())
}

fn validate_candidate_scores(candidate: &MemoryRetrievalCandidate) -> Result<(), MemoryError> {
    validate_score("lexical_score", candidate.lexical_score)?;
    validate_score("combined_score", candidate.combined_score)?;
    if let Some(semantic_score) = candidate.semantic_score {
        validate_score("semantic_score", semantic_score)?;
    }
    Ok(())
}

fn validate_score(label: &str, value: f32) -> Result<(), MemoryError> {
    if !value.is_finite() || value < 0.0 {
        return Err(invalid_rerank(format!(
            "{label} must be finite and non-negative"
        )));
    }
    Ok(())
}

fn compare_ranked_candidates(left: &RankedCandidate, right: &RankedCandidate) -> Ordering {
    right
        .candidate
        .combined_score
        .total_cmp(&left.candidate.combined_score)
        .then_with(|| {
            right
                .candidate
                .lexical_score
                .total_cmp(&left.candidate.lexical_score)
        })
        .then_with(|| {
            right
                .candidate
                .semantic_score
                .unwrap_or(0.0)
                .total_cmp(&left.candidate.semantic_score.unwrap_or(0.0))
        })
        .then_with(|| left.original_index.cmp(&right.original_index))
        .then_with(|| left.candidate.record.id.cmp(&right.candidate.record.id))
}

fn reject(
    candidate: MemoryRetrievalCandidate,
    rank: usize,
    token_estimate: u32,
    reason: MemoryRetrievalReason,
) -> MemoryRejectedCandidate {
    MemoryRejectedCandidate {
        candidate,
        rank,
        token_estimate,
        reasons: vec![reason],
    }
}

fn estimate_memory_tokens(record: &MemoryRecord) -> u32 {
    let count = record
        .text
        .split_whitespace()
        .filter(|word| !word.is_empty())
        .count();
    u32::try_from(count.max(1)).unwrap_or(u32::MAX)
}

fn invalid_rerank(message: impl Into<String>) -> MemoryError {
    MemoryError::InvalidRerank(redact_secret_like_text(&message.into()))
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
