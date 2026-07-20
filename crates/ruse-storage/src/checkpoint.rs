use ruse_types::{CheckpointId, ConversationId, EventSeq, SchemaVersion, Timestamp, TurnId};
use serde::{Deserialize, Serialize};

use crate::event_store::StorageError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub checkpoint_id: CheckpointId,
    pub conversation_id: ConversationId,
    pub turn_id: Option<TurnId>,
    pub label: String,
    pub event_seq: EventSeq,
    pub state_json: serde_json::Value,
    pub schema_version: SchemaVersion,
    pub created_at: Timestamp,
}

pub trait CheckpointStore {
    fn put_checkpoint(&mut self, checkpoint: CheckpointRecord) -> Result<(), StorageError>;

    fn latest_checkpoint(
        &self,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> Result<Option<CheckpointRecord>, StorageError>;
}

pub(crate) fn validate_checkpoint(checkpoint: &CheckpointRecord) -> Result<(), StorageError> {
    if checkpoint.label.trim().is_empty() {
        return Err(StorageError::InvalidCheckpoint(
            "checkpoint label cannot be empty".to_string(),
        ));
    }
    Ok(())
}
