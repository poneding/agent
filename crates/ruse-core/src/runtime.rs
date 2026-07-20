use ruse_types::{ConversationId, CorrelationId, TurnId, WorkspaceId};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RuntimeIds {
    pub workspace_id: WorkspaceId,
    pub conversation_id: ConversationId,
    pub turn_id: TurnId,
    pub correlation_id: CorrelationId,
}
