use agent_types::{ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};
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
