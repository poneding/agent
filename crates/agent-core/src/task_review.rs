use agent_scheduler::redact_task_text;
use agent_types::{CheckpointId, TaskId, TaskState};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const ARTIFACT_SUMMARY_LIMIT: usize = 160;
const REVIEW_SUMMARY_LIMIT: usize = 240;
const DIAGNOSTIC_LIMIT: usize = 160;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskArtifactKind {
    Report,
    Trace,
    Checkpoint,
    Transcript,
    Diff,
    Log,
    File,
    Other,
}

impl TaskArtifactKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Report => "report",
            Self::Trace => "trace",
            Self::Checkpoint => "checkpoint",
            Self::Transcript => "transcript",
            Self::Diff => "diff",
            Self::Log => "log",
            Self::File => "file",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskArtifact {
    pub artifact_id: String,
    pub task_id: TaskId,
    pub kind: TaskArtifactKind,
    pub path: Option<String>,
    pub content_hash: Option<String>,
    pub summary: String,
    pub missing: bool,
    pub diagnostics: Vec<String>,
}

impl TaskArtifact {
    pub fn new(
        artifact_id: impl Into<String>,
        task_id: TaskId,
        kind: TaskArtifactKind,
        path: Option<&str>,
        content_hash: Option<&str>,
        summary: impl Into<String>,
    ) -> Result<Self, TaskReviewError> {
        let artifact_id = normalize_required("artifact id", artifact_id.into())?;
        let summary = normalize_required("artifact summary", summary.into())?;

        Ok(Self {
            artifact_id,
            task_id,
            kind,
            path: normalize_optional(path, ARTIFACT_SUMMARY_LIMIT),
            content_hash: normalize_optional(content_hash, ARTIFACT_SUMMARY_LIMIT),
            summary: bounded_redacted(&summary, ARTIFACT_SUMMARY_LIMIT),
            missing: false,
            diagnostics: Vec::new(),
        })
    }

    pub fn missing(
        artifact_id: impl Into<String>,
        task_id: TaskId,
        kind: TaskArtifactKind,
        summary: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> Result<Self, TaskReviewError> {
        let mut artifact = Self::new(artifact_id, task_id, kind, None, None, summary)?;
        artifact.missing = true;
        artifact.diagnostics = vec![
            normalize_required("artifact diagnostic", diagnostic.into())
                .map(|value| bounded_redacted(&value, DIAGNOSTIC_LIMIT))?,
        ];
        Ok(artifact)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskReviewSummary {
    pub task_id: TaskId,
    pub final_state: TaskState,
    pub trace_id: Option<String>,
    pub checkpoint_id: Option<CheckpointId>,
    pub artifact_ids: Vec<String>,
    pub summary: String,
    pub diagnostics: Vec<String>,
}

impl TaskReviewSummary {
    pub fn new(
        task_id: TaskId,
        final_state: TaskState,
        trace_id: Option<&str>,
        checkpoint_id: Option<CheckpointId>,
        artifact_ids: Vec<String>,
        summary: impl Into<String>,
    ) -> Result<Self, TaskReviewError> {
        let summary = normalize_required("task review summary", summary.into())?;
        let artifact_ids = artifact_ids
            .into_iter()
            .map(|artifact_id| normalize_required("task review artifact id", artifact_id))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            task_id,
            final_state,
            trace_id: normalize_optional(trace_id, ARTIFACT_SUMMARY_LIMIT),
            checkpoint_id,
            artifact_ids,
            summary: bounded_redacted(&summary, REVIEW_SUMMARY_LIMIT),
            diagnostics: Vec::new(),
        })
    }

    pub fn from_artifacts(
        task_id: TaskId,
        final_state: TaskState,
        trace_id: Option<&str>,
        checkpoint_id: Option<CheckpointId>,
        artifacts: &[TaskArtifact],
        summary: impl Into<String>,
    ) -> Result<Self, TaskReviewError> {
        let mut review = Self::new(
            task_id,
            final_state,
            trace_id,
            checkpoint_id,
            artifacts
                .iter()
                .map(|artifact| artifact.artifact_id.clone())
                .collect(),
            summary,
        )?;

        review.diagnostics = artifacts
            .iter()
            .filter(|artifact| artifact.missing || artifact.task_id != task_id)
            .flat_map(|artifact| {
                let mut diagnostics = artifact.diagnostics.clone();
                if artifact.task_id != task_id {
                    diagnostics.push(format!(
                        "artifact {} belongs to a different task",
                        artifact.artifact_id
                    ));
                }
                diagnostics
            })
            .map(|diagnostic| bounded_redacted(&diagnostic, DIAGNOSTIC_LIMIT))
            .collect();

        Ok(review)
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TaskReviewError {
    #[error("invalid task review: {0}")]
    Validation(String),
}

fn normalize_required(label: &str, value: String) -> Result<String, TaskReviewError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TaskReviewError::Validation(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(bounded_redacted(value, REVIEW_SUMMARY_LIMIT))
}

fn normalize_optional(value: Option<&str>, limit: usize) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(bounded_redacted(value, limit))
        }
    })
}

fn bounded_redacted(value: &str, limit: usize) -> String {
    let redacted = redact_task_text(value);
    if redacted.chars().count() > limit {
        redacted.chars().take(limit).collect()
    } else {
        redacted
    }
}
