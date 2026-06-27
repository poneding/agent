use std::collections::BTreeMap;

use agent_types::EventId;
use serde::{Deserialize, Serialize};

use crate::{MemoryError, MemoryRecord, MemoryScope, MemorySensitivity, MemorySource};

pub trait MemoryProposalStore {
    fn add_proposal(&mut self, proposal: MemoryWriteProposal) -> Result<(), MemoryError>;

    fn list_proposals(&self) -> Vec<MemoryWriteProposal>;

    fn inspect_proposal(&self, proposal_id: &str) -> Option<MemoryWriteProposal>;

    fn resolve(
        &mut self,
        proposal_id: &str,
        decision: MemoryProposalDecision,
    ) -> Result<MemoryWriteProposal, MemoryError>;

    fn commit(&mut self, proposal_id: &str) -> Result<MemoryRecord, MemoryError>;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryProposalStatus {
    Pending,
    Approved,
    Rejected,
    Committed,
    Expired,
}

impl MemoryProposalStatus {
    const fn is_active(self) -> bool {
        matches!(self, Self::Pending | Self::Approved)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum MemoryProposalDecision {
    Approve,
    Reject { reason: String },
    Expire { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryWriteProposal {
    pub proposal_id: String,
    pub proposed_record_id: String,
    pub scope: MemoryScope,
    pub content: String,
    pub summary: String,
    pub source_event_ids: Vec<EventId>,
    pub confidence: u8,
    pub sensitivity: MemorySensitivity,
    pub status: MemoryProposalStatus,
    pub resolution_reason: Option<String>,
}

impl MemoryWriteProposal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        proposal_id: impl Into<String>,
        proposed_record_id: impl Into<String>,
        scope: MemoryScope,
        content: impl Into<String>,
        summary: impl Into<String>,
        source_event_ids: Vec<EventId>,
        confidence: u8,
        sensitivity: MemorySensitivity,
    ) -> Result<Self, MemoryError> {
        let proposal = Self {
            proposal_id: proposal_id.into().trim().to_string(),
            proposed_record_id: proposed_record_id.into().trim().to_string(),
            scope,
            content: content.into().trim().to_string(),
            summary: summary.into().trim().to_string(),
            source_event_ids,
            confidence,
            sensitivity,
            status: MemoryProposalStatus::Pending,
            resolution_reason: None,
        };
        validate_proposal(&proposal)?;
        Ok(proposal)
    }

    #[must_use]
    pub fn requires_explicit_approval(&self) -> bool {
        self.sensitivity != MemorySensitivity::Public
            || !matches!(self.scope, MemoryScope::Project { .. })
    }

    #[must_use]
    pub fn auto_approval_eligibility(&self) -> MemoryAutoApprovalEligibility {
        if self.status != MemoryProposalStatus::Pending {
            return MemoryAutoApprovalEligibility::not_eligible("proposal is not pending");
        }
        if self.requires_explicit_approval() {
            return MemoryAutoApprovalEligibility::not_eligible(
                "proposal requires explicit approval",
            );
        }
        if self.confidence < 80 {
            return MemoryAutoApprovalEligibility::not_eligible(
                "proposal confidence is below auto-approval threshold",
            );
        }
        if contains_secret_like_text(&self.content) || contains_secret_like_text(&self.summary) {
            return MemoryAutoApprovalEligibility::not_eligible(
                "proposal contains secret-like text",
            );
        }
        MemoryAutoApprovalEligibility {
            eligible: true,
            reason: "public project memory with sufficient confidence".to_string(),
        }
    }

    #[must_use]
    pub fn safe_summary(&self) -> MemoryProposalSummary {
        MemoryProposalSummary {
            proposal_id: redact_secret_like_text(&self.proposal_id),
            proposed_record_id: redact_secret_like_text(&self.proposed_record_id),
            scope: self.scope.clone(),
            summary: redact_secret_like_text(&self.summary),
            confidence: self.confidence,
            sensitivity: self.sensitivity,
            status: self.status,
            content_redacted: true,
            requires_explicit_approval: self.requires_explicit_approval(),
            auto_approval: self.auto_approval_eligibility(),
        }
    }

    fn to_memory_record(&self) -> Result<MemoryRecord, MemoryError> {
        validate_proposal(self)?;
        Ok(MemoryRecord {
            id: self.proposed_record_id.clone(),
            scope: self.scope.clone(),
            text: self.content.clone(),
            source: MemorySource {
                event_id: self.source_event_ids.first().copied(),
                description: self.summary.clone(),
            },
            confidence: self.confidence,
            sensitivity: self.sensitivity,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryAutoApprovalEligibility {
    pub eligible: bool,
    pub reason: String,
}

impl MemoryAutoApprovalEligibility {
    fn not_eligible(reason: impl Into<String>) -> Self {
        Self {
            eligible: false,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryProposalSummary {
    pub proposal_id: String,
    pub proposed_record_id: String,
    pub scope: MemoryScope,
    pub summary: String,
    pub confidence: u8,
    pub sensitivity: MemorySensitivity,
    pub status: MemoryProposalStatus,
    pub content_redacted: bool,
    pub requires_explicit_approval: bool,
    pub auto_approval: MemoryAutoApprovalEligibility,
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryMemoryProposalStore {
    proposals: BTreeMap<String, StoredProposal>,
}

impl MemoryProposalStore for InMemoryMemoryProposalStore {
    fn add_proposal(&mut self, proposal: MemoryWriteProposal) -> Result<(), MemoryError> {
        validate_proposal(&proposal)?;
        if proposal.status != MemoryProposalStatus::Pending {
            return Err(invalid_proposal(
                "only pending memory proposals can be added",
            ));
        }
        if self.proposals.contains_key(&proposal.proposal_id) {
            return Err(invalid_proposal(format!(
                "duplicate proposal id: {}",
                redact_secret_like_text(&proposal.proposal_id)
            )));
        }
        if self.proposals.values().any(|stored| {
            stored.proposal.status.is_active()
                && stored.proposal.proposed_record_id == proposal.proposed_record_id
        }) {
            return Err(invalid_proposal(format!(
                "duplicate proposed record id: {}",
                redact_secret_like_text(&proposal.proposed_record_id)
            )));
        }

        self.proposals.insert(
            proposal.proposal_id.clone(),
            StoredProposal {
                proposal,
                committed_record: None,
            },
        );
        Ok(())
    }

    fn list_proposals(&self) -> Vec<MemoryWriteProposal> {
        self.proposals
            .values()
            .map(|stored| stored.proposal.clone())
            .collect()
    }

    fn inspect_proposal(&self, proposal_id: &str) -> Option<MemoryWriteProposal> {
        self.proposals
            .get(proposal_id)
            .map(|stored| stored.proposal.clone())
    }

    fn resolve(
        &mut self,
        proposal_id: &str,
        decision: MemoryProposalDecision,
    ) -> Result<MemoryWriteProposal, MemoryError> {
        let stored = self.proposal_mut(proposal_id)?;
        if stored.proposal.status != MemoryProposalStatus::Pending {
            return Err(invalid_proposal(format!(
                "only pending proposals can be resolved: {}",
                redact_secret_like_text(proposal_id)
            )));
        }

        match decision {
            MemoryProposalDecision::Approve => {
                stored.proposal.status = MemoryProposalStatus::Approved;
                stored.proposal.resolution_reason = None;
            }
            MemoryProposalDecision::Reject { reason } => {
                stored.proposal.status = MemoryProposalStatus::Rejected;
                stored.proposal.resolution_reason = Some(redact_secret_like_text(&reason));
            }
            MemoryProposalDecision::Expire { reason } => {
                stored.proposal.status = MemoryProposalStatus::Expired;
                stored.proposal.resolution_reason = Some(redact_secret_like_text(&reason));
            }
        }
        Ok(stored.proposal.clone())
    }

    fn commit(&mut self, proposal_id: &str) -> Result<MemoryRecord, MemoryError> {
        let stored = self.proposal_mut(proposal_id)?;
        if stored.proposal.status == MemoryProposalStatus::Committed {
            return stored.committed_record.clone().ok_or_else(|| {
                invalid_proposal(format!(
                    "committed proposal is missing committed record: {}",
                    redact_secret_like_text(proposal_id)
                ))
            });
        }
        if stored.proposal.status != MemoryProposalStatus::Approved {
            return Err(invalid_proposal(format!(
                "only approved proposals can be committed: {}",
                redact_secret_like_text(proposal_id)
            )));
        }

        let record = stored.proposal.to_memory_record()?;
        stored.proposal.status = MemoryProposalStatus::Committed;
        stored.committed_record = Some(record.clone());
        Ok(record)
    }
}

impl InMemoryMemoryProposalStore {
    fn proposal_mut(&mut self, proposal_id: &str) -> Result<&mut StoredProposal, MemoryError> {
        self.proposals.get_mut(proposal_id).ok_or_else(|| {
            invalid_proposal(format!(
                "memory proposal not found: {}",
                redact_secret_like_text(proposal_id)
            ))
        })
    }
}

#[derive(Clone, Debug)]
struct StoredProposal {
    proposal: MemoryWriteProposal,
    committed_record: Option<MemoryRecord>,
}

fn validate_proposal(proposal: &MemoryWriteProposal) -> Result<(), MemoryError> {
    ensure_not_empty(&proposal.proposal_id, "proposal id")?;
    ensure_not_empty(&proposal.proposed_record_id, "proposed record id")?;
    ensure_not_empty(&proposal.content, "proposal content")?;
    ensure_not_empty(&proposal.summary, "proposal summary")?;
    if proposal.source_event_ids.is_empty() {
        return Err(invalid_proposal(
            "proposal source event ids cannot be empty",
        ));
    }
    if !(1..=100).contains(&proposal.confidence) {
        return Err(invalid_proposal(
            "proposal confidence must be between 1 and 100",
        ));
    }
    Ok(())
}

fn ensure_not_empty(value: &str, field: &str) -> Result<(), MemoryError> {
    if value.trim().is_empty() {
        return Err(invalid_proposal(format!("{field} cannot be empty")));
    }
    Ok(())
}

fn invalid_proposal(message: impl Into<String>) -> MemoryError {
    MemoryError::InvalidProposal(redact_secret_like_text(&message.into()))
}

fn contains_secret_like_text(value: &str) -> bool {
    value.split_whitespace().any(is_secret_like)
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
