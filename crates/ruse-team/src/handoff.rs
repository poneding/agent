use ruse_types::{AgentId, TaskId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{CapabilitySet, TeamError};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubagentRole {
    Implementer,
    Reviewer,
    Researcher,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubagentProfile {
    pub agent_id: AgentId,
    pub role: SubagentRole,
    pub display_name: String,
    pub system_prompt_summary: String,
}

impl SubagentProfile {
    pub fn new(
        agent_id: AgentId,
        role: SubagentRole,
        display_name: impl Into<String>,
        system_prompt_summary: impl Into<String>,
    ) -> Result<Self, TeamError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(TeamError::InvalidHandoff(
                "subagent display name cannot be empty".to_string(),
            ));
        }

        let system_prompt_summary = system_prompt_summary.into();
        if system_prompt_summary.trim().is_empty() {
            return Err(TeamError::InvalidHandoff(
                "subagent system prompt summary cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            agent_id,
            role,
            display_name,
            system_prompt_summary,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffStatus {
    Created,
    Accepted,
    Completed,
    Failed,
    Cancelled,
}

impl HandoffStatus {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HandoffEnvelope {
    pub task_id: TaskId,
    pub parent_agent_id: AgentId,
    pub subagent_profile: SubagentProfile,
    pub worktree_lease_id: String,
    pub capability_set: CapabilitySet,
    pub status: HandoffStatus,
    pub objective: String,
    pub created_at: Timestamp,
}

impl HandoffEnvelope {
    pub fn new(
        task_id: TaskId,
        parent_agent_id: AgentId,
        subagent_profile: SubagentProfile,
        worktree_lease_id: impl Into<String>,
        capability_set: CapabilitySet,
        objective: impl Into<String>,
        created_at: Timestamp,
    ) -> Result<Self, TeamError> {
        let worktree_lease_id = worktree_lease_id.into();
        if worktree_lease_id.trim().is_empty() {
            return Err(TeamError::InvalidHandoff(
                "worktree lease id cannot be empty".to_string(),
            ));
        }

        if capability_set.is_empty() {
            return Err(TeamError::InvalidHandoff(
                "handoff capability set cannot be empty".to_string(),
            ));
        }

        let objective = objective.into();
        if objective.trim().is_empty() {
            return Err(TeamError::InvalidHandoff(
                "handoff objective cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            task_id,
            parent_agent_id,
            subagent_profile,
            worktree_lease_id,
            capability_set,
            status: HandoffStatus::Created,
            objective,
            created_at,
        })
    }
}

pub fn transition_handoff_status(
    from: HandoffStatus,
    to: HandoffStatus,
    reason: impl AsRef<str>,
) -> Result<HandoffStatus, TeamError> {
    if reason.as_ref().trim().is_empty() {
        return Err(TeamError::InvalidTransition(
            "handoff transition reason cannot be empty".to_string(),
        ));
    }

    if from.is_terminal() {
        return Err(TeamError::InvalidTransition(format!(
            "terminal handoff status {from:?} cannot transition"
        )));
    }

    if !is_allowed_transition(from, to) {
        return Err(TeamError::InvalidTransition(format!(
            "handoff cannot transition from {from:?} to {to:?}"
        )));
    }

    Ok(to)
}

fn is_allowed_transition(from: HandoffStatus, to: HandoffStatus) -> bool {
    match from {
        HandoffStatus::Created => matches!(
            to,
            HandoffStatus::Accepted | HandoffStatus::Failed | HandoffStatus::Cancelled
        ),
        HandoffStatus::Accepted => matches!(
            to,
            HandoffStatus::Completed | HandoffStatus::Failed | HandoffStatus::Cancelled
        ),
        HandoffStatus::Completed | HandoffStatus::Failed | HandoffStatus::Cancelled => false,
    }
}
