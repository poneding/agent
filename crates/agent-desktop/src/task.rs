use agent_types::{ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};
use agent_ui_bridge::{UiTaskReviewSummary, UiTaskReviewSummaryArtifact, UiTaskSummary};
use serde::{Deserialize, Serialize};

use crate::DesktopError;
use crate::conversation::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskItem {
    pub task_id: TaskId,
    pub workspace_id: WorkspaceId,
    pub conversation_id: Option<ConversationId>,
    pub state: TaskState,
    pub summary: String,
    pub updated_at: Timestamp,
}

impl DesktopTaskItem {
    pub fn new(
        task_id: TaskId,
        workspace_id: WorkspaceId,
        conversation_id: Option<ConversationId>,
        state: TaskState,
        summary: impl Into<String>,
        updated_at: Timestamp,
    ) -> Result<Self, DesktopError> {
        Ok(Self {
            task_id,
            workspace_id,
            conversation_id,
            state,
            summary: validate_non_empty("task summary", summary.into())?,
            updated_at,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("task summary", self.summary.clone())?;
        Ok(())
    }
}

impl TryFrom<&UiTaskSummary> for DesktopTaskItem {
    type Error = DesktopError;

    fn try_from(summary: &UiTaskSummary) -> Result<Self, Self::Error> {
        Self::new(
            summary.task_id,
            summary.workspace_id,
            summary.conversation_id,
            summary.state,
            summary.prompt_summary.clone(),
            summary.updated_at,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskReviewItem {
    pub task_id: TaskId,
    pub final_state: TaskState,
    pub summary: String,
    pub artifact_count: u32,
    pub trace_id: Option<String>,
    pub artifacts: Vec<DesktopTaskArtifactItem>,
    pub diagnostics: Vec<String>,
}

impl DesktopTaskReviewItem {
    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("task review summary", self.summary.clone())?;
        for artifact in &self.artifacts {
            artifact.validate()?;
        }
        Ok(())
    }
}

impl TryFrom<&UiTaskReviewSummary> for DesktopTaskReviewItem {
    type Error = DesktopError;

    fn try_from(summary: &UiTaskReviewSummary) -> Result<Self, Self::Error> {
        let item = Self {
            task_id: summary.task_id,
            final_state: summary.final_state,
            summary: validate_non_empty("task review summary", summary.summary.clone())?,
            artifact_count: summary.artifact_count,
            trace_id: summary.trace_id.clone(),
            artifacts: summary
                .artifacts
                .iter()
                .map(DesktopTaskArtifactItem::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            diagnostics: summary.diagnostics.clone(),
        };
        item.validate()?;
        Ok(item)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopTaskArtifactItem {
    pub artifact_id: String,
    pub kind: String,
    pub path: Option<String>,
    pub content_hash: Option<String>,
    pub summary: String,
    pub missing: bool,
    pub diagnostics: Vec<String>,
}

impl DesktopTaskArtifactItem {
    pub(crate) fn validate(&self) -> Result<(), DesktopError> {
        validate_non_empty("task artifact id", self.artifact_id.clone())?;
        validate_non_empty("task artifact kind", self.kind.clone())?;
        validate_non_empty("task artifact summary", self.summary.clone())?;
        Ok(())
    }
}

impl TryFrom<&UiTaskReviewSummaryArtifact> for DesktopTaskArtifactItem {
    type Error = DesktopError;

    fn try_from(artifact: &UiTaskReviewSummaryArtifact) -> Result<Self, Self::Error> {
        let item = Self {
            artifact_id: validate_non_empty("task artifact id", artifact.artifact_id.clone())?,
            kind: validate_non_empty("task artifact kind", artifact.kind.clone())?,
            path: artifact.path.clone(),
            content_hash: artifact.content_hash.clone(),
            summary: validate_non_empty("task artifact summary", artifact.summary.clone())?,
            missing: artifact.missing,
            diagnostics: artifact.diagnostics.clone(),
        };
        item.validate()?;
        Ok(item)
    }
}
