use agent_types::{AgentId, EventId, TaskId, TeamId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::TeamError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlackboardEntryKind {
    Finding,
    DiffSummary,
    Review,
    Decision,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub label: String,
    pub path: String,
    pub content_hash: Option<String>,
}

impl ArtifactRef {
    pub fn new(
        label: impl Into<String>,
        path: impl Into<String>,
        content_hash: Option<&str>,
    ) -> Result<Self, TeamError> {
        let label = validate_non_empty("artifact label", label.into())?;
        let path = validate_relative_path("artifact path", path.into())?;
        let content_hash = match content_hash {
            Some(value) if value.trim().is_empty() => {
                return Err(TeamError::InvalidBlackboard(
                    "artifact content hash cannot be empty".to_string(),
                ));
            }
            Some(value) => Some(value.trim().to_string()),
            None => None,
        };

        Ok(Self {
            label,
            path,
            content_hash,
        })
    }

    fn validate(&self) -> Result<(), TeamError> {
        validate_non_empty("artifact label", self.label.clone())?;
        validate_relative_path("artifact path", self.path.clone())?;
        if self
            .content_hash
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(TeamError::InvalidBlackboard(
                "artifact content hash cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVote {
    Approve,
    RequestChanges,
    Block,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlackboardEntry {
    pub team_id: TeamId,
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub kind: BlackboardEntryKind,
    pub summary: String,
    pub artifacts: Vec<ArtifactRef>,
    pub trace_event_ids: Vec<EventId>,
    pub review_vote: Option<ReviewVote>,
    pub created_at: Timestamp,
}

impl BlackboardEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        team_id: TeamId,
        agent_id: AgentId,
        task_id: TaskId,
        kind: BlackboardEntryKind,
        summary: impl Into<String>,
        artifacts: Vec<ArtifactRef>,
        trace_event_ids: Vec<EventId>,
        review_vote: Option<ReviewVote>,
        created_at: Timestamp,
    ) -> Result<Self, TeamError> {
        let entry = Self {
            team_id,
            agent_id,
            task_id,
            kind,
            summary: summary.into(),
            artifacts,
            trace_event_ids,
            review_vote,
            created_at,
        };
        entry.validate()?;
        Ok(entry)
    }

    fn validate(&self) -> Result<(), TeamError> {
        validate_non_empty("blackboard summary", self.summary.clone())?;
        for artifact in &self.artifacts {
            artifact.validate()?;
        }
        Ok(())
    }
}

pub trait BlackboardStore {
    fn append(&mut self, entry: BlackboardEntry) -> Result<(), TeamError>;
    fn list_for_team(&self, team_id: TeamId) -> Vec<BlackboardEntry>;
}

#[derive(Clone, Debug, Default)]
pub struct InMemoryBlackboardStore {
    entries: Vec<BlackboardEntry>,
}

impl BlackboardStore for InMemoryBlackboardStore {
    fn append(&mut self, entry: BlackboardEntry) -> Result<(), TeamError> {
        entry.validate()?;
        self.entries.push(entry);
        self.entries.sort_by_key(|entry| entry.created_at);
        Ok(())
    }

    fn list_for_team(&self, team_id: TeamId) -> Vec<BlackboardEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.team_id == team_id)
            .cloned()
            .collect()
    }
}

fn validate_non_empty(label: &str, value: String) -> Result<String, TeamError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(TeamError::InvalidBlackboard(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(value)
}

fn validate_relative_path(label: &str, value: String) -> Result<String, TeamError> {
    let value = value.trim().replace('\\', "/");
    if value.is_empty() {
        return Err(TeamError::InvalidBlackboard(format!(
            "{label} cannot be empty"
        )));
    }
    if value.starts_with('/') || value.split('/').any(|segment| segment == "..") {
        return Err(TeamError::InvalidBlackboard(format!(
            "{label} must stay relative to the workspace"
        )));
    }
    Ok(value)
}
