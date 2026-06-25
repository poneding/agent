use agent_types::{TaskId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::DesktopError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffReviewStatus {
    Pending,
    Approved,
    Rejected,
}

impl DiffReviewStatus {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Approved | Self::Rejected)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiffFileSummary {
    pub path: String,
    pub added_lines: u32,
    pub removed_lines: u32,
    pub summary: String,
}

impl DiffFileSummary {
    pub fn new(
        path: impl Into<String>,
        added_lines: u32,
        removed_lines: u32,
        summary: impl Into<String>,
    ) -> Result<Self, DesktopError> {
        Ok(Self {
            path: validate_relative_path("diff path", path.into())?,
            added_lines,
            removed_lines,
            summary: validate_non_empty("diff summary", summary.into())?,
        })
    }

    fn validate(&self) -> Result<(), DesktopError> {
        validate_relative_path("diff path", self.path.clone())?;
        validate_non_empty("diff summary", self.summary.clone())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiffReview {
    pub task_id: TaskId,
    pub files: Vec<DiffFileSummary>,
    pub status: DiffReviewStatus,
    pub created_at: Timestamp,
    pub decided_at: Option<Timestamp>,
}

impl DiffReview {
    pub fn new(
        task_id: TaskId,
        files: Vec<DiffFileSummary>,
        created_at: Timestamp,
    ) -> Result<Self, DesktopError> {
        if files.is_empty() {
            return Err(DesktopError::InvalidDiff(
                "diff review files cannot be empty".to_string(),
            ));
        }
        for file in &files {
            file.validate()?;
        }

        Ok(Self {
            task_id,
            files,
            status: DiffReviewStatus::Pending,
            created_at,
            decided_at: None,
        })
    }
}

pub fn approve_diff_review(
    review: &DiffReview,
    now: Timestamp,
) -> Result<DiffReview, DesktopError> {
    transition_diff_review(review, DiffReviewStatus::Approved, now)
}

pub fn reject_diff_review(review: &DiffReview, now: Timestamp) -> Result<DiffReview, DesktopError> {
    transition_diff_review(review, DiffReviewStatus::Rejected, now)
}

fn transition_diff_review(
    review: &DiffReview,
    status: DiffReviewStatus,
    now: Timestamp,
) -> Result<DiffReview, DesktopError> {
    if review.status.is_terminal() {
        return Err(DesktopError::InvalidDiff(format!(
            "terminal diff review {:?} cannot transition",
            review.status
        )));
    }
    if status == DiffReviewStatus::Pending {
        return Err(DesktopError::InvalidDiff(
            "diff review cannot transition back to pending".to_string(),
        ));
    }

    let mut next = review.clone();
    next.status = status;
    next.decided_at = Some(now);
    Ok(next)
}

fn validate_non_empty(label: &str, value: String) -> Result<String, DesktopError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DesktopError::InvalidDiff(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(value)
}

fn validate_relative_path(label: &str, value: String) -> Result<String, DesktopError> {
    let value = value.trim().replace('\\', "/");
    if value.is_empty() {
        return Err(DesktopError::InvalidDiff(format!(
            "{label} cannot be empty"
        )));
    }
    if value.starts_with('/') || value.split('/').any(|segment| segment == "..") {
        return Err(DesktopError::InvalidDiff(format!(
            "{label} must stay relative to the workspace"
        )));
    }
    Ok(value)
}
