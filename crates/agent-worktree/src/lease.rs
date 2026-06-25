use agent_types::{TaskId, Timestamp, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::WorktreeError;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorktreeLeaseId(String);

impl WorktreeLeaseId {
    pub fn new(value: impl Into<String>) -> Result<Self, WorktreeError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(WorktreeError::InvalidRequest(
                "worktree lease id cannot be empty".to_string(),
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub(crate) fn generated(sequence: u64) -> Self {
        Self(format!("worktree-{sequence}"))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeLeaseState {
    Planned,
    Active,
    Released,
    CleanupFailed,
}

impl WorktreeLeaseState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Released | Self::CleanupFailed)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeCleanupPolicy {
    DeleteOnRelease,
    KeepForReview,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorktreeRequest {
    pub workspace_id: WorkspaceId,
    pub task_id: TaskId,
    pub base_branch: String,
    pub worktree_branch: String,
    pub worktree_path: String,
    pub cleanup_policy: WorktreeCleanupPolicy,
}

impl WorktreeRequest {
    pub fn new(
        workspace_id: WorkspaceId,
        task_id: TaskId,
        base_branch: impl Into<String>,
        worktree_branch: impl Into<String>,
        worktree_path: impl Into<String>,
        cleanup_policy: WorktreeCleanupPolicy,
    ) -> Result<Self, WorktreeError> {
        let base_branch = validate_branch("base_branch", base_branch.into())?;
        let worktree_branch = validate_branch("worktree_branch", worktree_branch.into())?;
        let worktree_path = validate_relative_path(worktree_path.into())?;

        Ok(Self {
            workspace_id,
            task_id,
            base_branch,
            worktree_branch,
            worktree_path,
            cleanup_policy,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorktreeLease {
    pub id: WorktreeLeaseId,
    pub workspace_id: WorkspaceId,
    pub task_id: TaskId,
    pub base_branch: String,
    pub worktree_branch: String,
    pub worktree_path: String,
    pub state: WorktreeLeaseState,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl WorktreeLease {
    #[must_use]
    pub(crate) fn planned(id: WorktreeLeaseId, request: WorktreeRequest, now: Timestamp) -> Self {
        Self {
            id,
            workspace_id: request.workspace_id,
            task_id: request.task_id,
            base_branch: request.base_branch,
            worktree_branch: request.worktree_branch,
            worktree_path: request.worktree_path,
            state: WorktreeLeaseState::Planned,
            created_at: now,
            updated_at: now,
        }
    }
}

fn validate_branch(label: &str, value: String) -> Result<String, WorktreeError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(WorktreeError::InvalidRequest(format!(
            "{label} cannot be empty"
        )));
    }
    if value.contains(char::is_whitespace) || value.contains("..") {
        return Err(WorktreeError::InvalidRequest(format!(
            "{label} contains unsafe characters"
        )));
    }
    Ok(value)
}

fn validate_relative_path(value: String) -> Result<String, WorktreeError> {
    let value = value.trim().replace('\\', "/");
    if value.is_empty() {
        return Err(WorktreeError::InvalidRequest(
            "worktree path cannot be empty".to_string(),
        ));
    }
    if value.starts_with('/') || value.split('/').any(|segment| segment == "..") {
        return Err(WorktreeError::InvalidRequest(
            "worktree path must stay relative to the workspace".to_string(),
        ));
    }
    Ok(value)
}
