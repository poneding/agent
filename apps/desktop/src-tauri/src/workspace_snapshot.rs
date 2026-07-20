use ruse_desktop::{
    DesktopConversationItem, DesktopTaskItem, DesktopWorkspaceSnapshot, build_workspace_snapshot,
};
use ruse_types::{ConversationId, WorkspaceId};

use crate::bridge::DesktopRuntimeSession;
use crate::commands::DesktopCommandError;

pub fn build_empty_workspace_snapshot_for_session(
    session: &mut DesktopRuntimeSession,
) -> Result<DesktopWorkspaceSnapshot, DesktopCommandError> {
    let (workspace_id, conversation_id) = session.ensure_workspace_snapshot_ids();
    let conversations = session.persisted_conversations()?;
    build_workspace_snapshot_response(
        workspace_id,
        Some(conversation_id),
        conversations,
        Vec::new(),
    )
}

pub fn build_workspace_snapshot_response(
    workspace_id: WorkspaceId,
    active_conversation_id: Option<ConversationId>,
    conversations: Vec<DesktopConversationItem>,
    tasks: Vec<DesktopTaskItem>,
) -> Result<DesktopWorkspaceSnapshot, DesktopCommandError> {
    build_workspace_snapshot(workspace_id, active_conversation_id, conversations, tasks).map_err(
        |error| DesktopCommandError::new("desktop.workspace_snapshot.invalid", error.to_string()),
    )
}
