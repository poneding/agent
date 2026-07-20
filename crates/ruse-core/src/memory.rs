use ruse_context::{PromptLayer, memory_prompt_layers};
use ruse_memory::{
    DeterministicMemoryReranker, MemoryPrivacyDiagnostic, MemoryPrivacyDiagnosticSeverity,
    MemoryProductSnapshot, MemoryProposalSummary, MemoryRerankRequest, MemoryReranker,
    MemoryRetentionControlSummary, MemoryRetrievalCandidate, MemoryRetrievalResult, MemoryScope,
    MemorySensitivity, MemoryWriteProposal,
};
use ruse_types::EventId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DIAGNOSTIC_LIMIT_BYTES: usize = 160;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeMemoryConfig {
    pub enabled: bool,
    pub max_selected: usize,
    pub prompt_budget_tokens: Option<u32>,
    pub fail_closed: bool,
}

impl Default for RuntimeMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_selected: 3,
            prompt_budget_tokens: Some(256),
            fail_closed: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeMemoryProposalPolicy {
    pub enabled: bool,
}

impl Default for RuntimeMemoryProposalPolicy {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMemoryInput {
    pub query: String,
    pub candidates: Vec<MemoryRetrievalCandidate>,
    pub config: RuntimeMemoryConfig,
    pub proposal_policy: RuntimeMemoryProposalPolicy,
    pub proposal_inputs: Vec<RuntimeMemoryProposalInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeMemoryProposalInput {
    pub proposal_id: String,
    pub proposed_record_id: String,
    pub scope: MemoryScope,
    pub content: String,
    pub summary: String,
    pub source_event_ids: Vec<EventId>,
    pub confidence: u8,
    pub sensitivity: MemorySensitivity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMemoryAssembly {
    pub layers: Vec<PromptLayer>,
    pub evidence: RuntimeMemoryTurnEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeMemoryTurnEvidence {
    pub enabled: bool,
    pub layer_count: usize,
    pub influence: RuntimeMemoryInfluenceReport,
    pub proposals: RuntimeMemoryProposalReport,
    pub snapshot: MemoryProductSnapshot,
    pub diagnostics: Vec<RuntimeMemoryDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeMemoryInfluenceReport {
    pub query_hash: Option<String>,
    pub selected_count: usize,
    pub rejected_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeMemoryProposalReport {
    pub created_count: usize,
    pub skipped_count: usize,
    pub proposals: Vec<MemoryProposalSummary>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMemoryDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeMemoryDiagnostic {
    pub severity: RuntimeMemoryDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

impl RuntimeMemoryDiagnostic {
    #[must_use]
    pub fn new(
        severity: RuntimeMemoryDiagnosticSeverity,
        code: impl AsRef<str>,
        message: impl AsRef<str>,
    ) -> Self {
        Self {
            severity,
            code: safe_text(code.as_ref(), DIAGNOSTIC_LIMIT_BYTES),
            message: safe_text(message.as_ref(), DIAGNOSTIC_LIMIT_BYTES),
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RuntimeMemoryError {
    #[error("runtime memory failed: {message}")]
    Failed { message: String },
}

pub fn assemble_runtime_memory(
    input: RuntimeMemoryInput,
) -> Result<RuntimeMemoryAssembly, RuntimeMemoryError> {
    if !input.config.enabled {
        return Ok(empty_memory_assembly(
            false,
            vec![RuntimeMemoryDiagnostic::new(
                RuntimeMemoryDiagnosticSeverity::Info,
                "memory_disabled",
                "runtime memory is disabled",
            )],
        ));
    }

    let proposal_build = build_memory_proposals(
        input.proposal_policy,
        input.proposal_inputs,
        input.config.fail_closed,
    )?;
    let retrieval = match DeterministicMemoryReranker::default().rerank(MemoryRerankRequest {
        query_text: input.query,
        candidates: input.candidates.clone(),
        max_selected: input.config.max_selected,
        prompt_budget_tokens: input.config.prompt_budget_tokens,
    }) {
        Ok(retrieval) => retrieval,
        Err(error) => {
            return memory_failure(
                input.config.fail_closed,
                RuntimeMemoryDiagnostic::new(
                    RuntimeMemoryDiagnosticSeverity::Warning,
                    "memory_retrieval_failed",
                    error.to_string(),
                ),
            );
        }
    };

    let layers = match memory_prompt_layers(&retrieval.selected) {
        Ok(layers) => layers,
        Err(error) => {
            return memory_failure(
                input.config.fail_closed,
                RuntimeMemoryDiagnostic::new(
                    RuntimeMemoryDiagnosticSeverity::Warning,
                    "memory_layer_failed",
                    error.to_string(),
                ),
            );
        }
    };

    let records = input
        .candidates
        .iter()
        .map(|candidate| candidate.record.clone())
        .collect::<Vec<_>>();
    let snapshot = MemoryProductSnapshot::from_parts(
        "runtime memory",
        &records,
        &proposal_build.proposals,
        Some(&retrieval),
        Some(&retrieval.trace),
        MemoryRetentionControlSummary::default(),
        &[],
        proposal_build
            .diagnostics
            .iter()
            .map(memory_privacy_diagnostic)
            .collect(),
    );

    Ok(RuntimeMemoryAssembly {
        evidence: RuntimeMemoryTurnEvidence {
            enabled: true,
            layer_count: layers.len(),
            influence: influence_report(&retrieval),
            proposals: RuntimeMemoryProposalReport {
                created_count: proposal_build.proposals.len(),
                skipped_count: proposal_build.skipped_count,
                proposals: proposal_build
                    .proposals
                    .iter()
                    .map(MemoryWriteProposal::safe_summary)
                    .collect(),
            },
            snapshot,
            diagnostics: proposal_build.diagnostics,
        },
        layers,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RuntimeProposalBuild {
    proposals: Vec<MemoryWriteProposal>,
    skipped_count: usize,
    diagnostics: Vec<RuntimeMemoryDiagnostic>,
}

fn build_memory_proposals(
    policy: RuntimeMemoryProposalPolicy,
    proposal_inputs: Vec<RuntimeMemoryProposalInput>,
    fail_closed: bool,
) -> Result<RuntimeProposalBuild, RuntimeMemoryError> {
    if !policy.enabled {
        return Ok(RuntimeProposalBuild {
            proposals: Vec::new(),
            skipped_count: proposal_inputs.len(),
            diagnostics: if proposal_inputs.is_empty() {
                Vec::new()
            } else {
                vec![RuntimeMemoryDiagnostic::new(
                    RuntimeMemoryDiagnosticSeverity::Info,
                    "memory_proposals_disabled",
                    "runtime memory proposals are disabled",
                )]
            },
        });
    }

    let mut proposals = Vec::new();
    let mut diagnostics = Vec::new();
    let mut skipped_count = 0_usize;
    for input in proposal_inputs {
        match MemoryWriteProposal::new(
            input.proposal_id,
            input.proposed_record_id,
            input.scope,
            input.content,
            input.summary,
            input.source_event_ids,
            input.confidence,
            input.sensitivity,
        ) {
            Ok(proposal) => proposals.push(proposal),
            Err(error) => {
                skipped_count += 1;
                let diagnostic = RuntimeMemoryDiagnostic::new(
                    RuntimeMemoryDiagnosticSeverity::Warning,
                    "memory_proposal_invalid",
                    error.to_string(),
                );
                if fail_closed {
                    return Err(RuntimeMemoryError::Failed {
                        message: diagnostic.message,
                    });
                }
                diagnostics.push(diagnostic);
            }
        }
    }

    Ok(RuntimeProposalBuild {
        proposals,
        skipped_count,
        diagnostics,
    })
}

fn empty_memory_assembly(
    enabled: bool,
    diagnostics: Vec<RuntimeMemoryDiagnostic>,
) -> RuntimeMemoryAssembly {
    let snapshot = MemoryProductSnapshot::from_parts(
        "runtime memory",
        &[],
        &[],
        None,
        None,
        MemoryRetentionControlSummary::default(),
        &[],
        diagnostics.iter().map(memory_privacy_diagnostic).collect(),
    );
    RuntimeMemoryAssembly {
        layers: Vec::new(),
        evidence: RuntimeMemoryTurnEvidence {
            enabled,
            layer_count: 0,
            influence: RuntimeMemoryInfluenceReport {
                query_hash: None,
                selected_count: 0,
                rejected_count: 0,
            },
            proposals: RuntimeMemoryProposalReport {
                created_count: 0,
                skipped_count: 0,
                proposals: Vec::new(),
            },
            snapshot,
            diagnostics,
        },
    }
}

fn memory_failure(
    fail_closed: bool,
    diagnostic: RuntimeMemoryDiagnostic,
) -> Result<RuntimeMemoryAssembly, RuntimeMemoryError> {
    if fail_closed {
        return Err(RuntimeMemoryError::Failed {
            message: diagnostic.message,
        });
    }
    Ok(empty_memory_assembly(true, vec![diagnostic]))
}

fn influence_report(retrieval: &MemoryRetrievalResult) -> RuntimeMemoryInfluenceReport {
    RuntimeMemoryInfluenceReport {
        query_hash: Some(retrieval.trace.query_hash.clone()),
        selected_count: retrieval.selected.len(),
        rejected_count: retrieval.rejected.len(),
    }
}

fn memory_privacy_diagnostic(diagnostic: &RuntimeMemoryDiagnostic) -> MemoryPrivacyDiagnostic {
    let severity = match diagnostic.severity {
        RuntimeMemoryDiagnosticSeverity::Info => MemoryPrivacyDiagnosticSeverity::Info,
        RuntimeMemoryDiagnosticSeverity::Warning => MemoryPrivacyDiagnosticSeverity::Warning,
        RuntimeMemoryDiagnosticSeverity::Error => MemoryPrivacyDiagnosticSeverity::Blocked,
    };
    MemoryPrivacyDiagnostic::new(&diagnostic.code, severity, &diagnostic.message)
}

fn safe_text(value: &str, max_bytes: usize) -> String {
    let redacted = redact_secret_like_text(value);
    truncate_utf8(&redacted, max_bytes)
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
