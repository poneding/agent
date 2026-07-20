use std::cmp::Reverse;

use ruse_types::{ConversationId, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::{DesktopConversationItem, DesktopError, DesktopTaskItem};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopWorkspaceSnapshot {
    pub workspace_id: WorkspaceId,
    pub active_conversation_id: Option<ConversationId>,
    pub conversations: Vec<DesktopConversationItem>,
    pub tasks: Vec<DesktopTaskItem>,
}

pub fn build_workspace_snapshot(
    workspace_id: WorkspaceId,
    active_conversation_id: Option<ConversationId>,
    mut conversations: Vec<DesktopConversationItem>,
    mut tasks: Vec<DesktopTaskItem>,
) -> Result<DesktopWorkspaceSnapshot, DesktopError> {
    for conversation in &conversations {
        conversation.validate()?;
    }
    for task in &tasks {
        task.validate()?;
    }

    conversations.sort_by_key(|conversation| {
        (
            Reverse(conversation.last_activity_at),
            conversation.title.clone(),
        )
    });
    tasks.sort_by_key(|task| {
        (
            task.state.is_terminal(),
            Reverse(task.updated_at),
            task.summary.clone(),
        )
    });

    Ok(DesktopWorkspaceSnapshot {
        workspace_id,
        active_conversation_id,
        conversations,
        tasks,
    })
}
