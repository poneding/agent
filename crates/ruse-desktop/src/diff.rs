use ruse_types::{TaskId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::DesktopError;

const MAX_DIFF_TEXT_CHARS: usize = 240;
const MAX_DIFF_HUNK_LINES: usize = 80;

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopDiffReview {
    pub review_id: String,
    pub status: DiffReviewStatus,
    pub files: Vec<DesktopDiffFileReview>,
    pub selected_path: Option<String>,
    pub decision_enabled: bool,
    pub disabled_reason: Option<String>,
}

impl DesktopDiffReview {
    pub fn new(
        review_id: impl Into<String>,
        status: DiffReviewStatus,
        files: Vec<DesktopDiffFileReview>,
        selected_path: Option<String>,
    ) -> Result<Self, DesktopError> {
        if files.is_empty() {
            return Err(DesktopError::InvalidDiff(
                "desktop diff review files cannot be empty".to_string(),
            ));
        }
        for file in &files {
            file.validate()?;
        }
        let selected_path = selected_path
            .map(|path| validate_relative_path("selected diff path", path))
            .transpose()?;
        if let Some(selected_path) = &selected_path
            && !files.iter().any(|file| &file.path == selected_path)
        {
            return Err(DesktopError::InvalidDiff(
                "selected diff path must exist in review files".to_string(),
            ));
        }
        let disabled_reason = disabled_reason_for_status(status);
        Ok(Self {
            review_id: validate_non_empty("desktop diff review id", review_id.into())?,
            status,
            files,
            selected_path,
            decision_enabled: disabled_reason.is_none(),
            disabled_reason,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopDiffFileReview {
    pub path: String,
    pub added_lines: u32,
    pub removed_lines: u32,
    pub summary: String,
    pub hunks: Vec<DesktopDiffHunkPreview>,
}

impl DesktopDiffFileReview {
    pub fn new(
        path: impl Into<String>,
        added_lines: u32,
        removed_lines: u32,
        summary: impl Into<String>,
        hunks: Vec<DesktopDiffHunkPreview>,
    ) -> Result<Self, DesktopError> {
        Ok(Self {
            path: validate_relative_path("desktop diff file path", path.into())?,
            added_lines,
            removed_lines,
            summary: validate_bounded_text("desktop diff file summary", summary.into())?.0,
            hunks,
        })
    }

    fn validate(&self) -> Result<(), DesktopError> {
        validate_relative_path("desktop diff file path", self.path.clone())?;
        validate_non_empty("desktop diff file summary", self.summary.clone())?;
        for hunk in &self.hunks {
            hunk.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopDiffHunkPreview {
    pub old_start: u32,
    pub new_start: u32,
    pub lines: Vec<DesktopDiffLinePreview>,
    pub truncated: bool,
}

impl DesktopDiffHunkPreview {
    pub fn new(
        old_start: u32,
        new_start: u32,
        lines: Vec<DesktopDiffLinePreview>,
    ) -> Result<Self, DesktopError> {
        if lines.is_empty() {
            return Err(DesktopError::InvalidDiff(
                "desktop diff hunk lines cannot be empty".to_string(),
            ));
        }
        let truncated =
            lines.len() > MAX_DIFF_HUNK_LINES || lines.iter().any(|line| line.truncated);
        let lines = lines.into_iter().take(MAX_DIFF_HUNK_LINES).collect();
        Ok(Self {
            old_start,
            new_start,
            lines,
            truncated,
        })
    }

    fn validate(&self) -> Result<(), DesktopError> {
        if self.lines.is_empty() {
            return Err(DesktopError::InvalidDiff(
                "desktop diff hunk lines cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopDiffLinePreview {
    pub kind: DesktopDiffLineKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub content: String,
    pub truncated: bool,
}

impl DesktopDiffLinePreview {
    pub fn new(
        kind: DesktopDiffLineKind,
        old_line: Option<u32>,
        new_line: Option<u32>,
        content: impl Into<String>,
    ) -> Result<Self, DesktopError> {
        let (content, truncated) = validate_bounded_text("desktop diff line", content.into())?;
        Ok(Self {
            kind,
            old_line,
            new_line,
            content,
            truncated,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopDiffLineKind {
    Context,
    Addition,
    Removal,
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

fn validate_bounded_text(label: &str, value: String) -> Result<(String, bool), DesktopError> {
    let value = validate_non_empty(label, value)?;
    Ok(bound_diff_text(&value))
}

fn bound_diff_text(value: &str) -> (String, bool) {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() > MAX_DIFF_TEXT_CHARS {
        (redacted.chars().take(MAX_DIFF_TEXT_CHARS).collect(), true)
    } else {
        (redacted, false)
    }
}

fn disabled_reason_for_status(status: DiffReviewStatus) -> Option<String> {
    match status {
        DiffReviewStatus::Pending => None,
        DiffReviewStatus::Approved => Some("diff review already approved".to_string()),
        DiffReviewStatus::Rejected => Some("diff review already rejected".to_string()),
    }
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
    value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
        || value.starts_with("sk-")
}
