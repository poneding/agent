use std::collections::{BTreeMap, HashMap};

use agent_types::{CheckpointId, ConversationId, EventId, RuntimeEvent, TurnId};

use crate::checkpoint::{CheckpointRecord, CheckpointStore, validate_checkpoint};
use crate::event_store::{EventStore, StorageError, StoredEvent, assign_sequence};

#[derive(Default)]
pub struct InMemoryEventStore {
    events: HashMap<EventId, StoredEvent>,
    streams: BTreeMap<ConversationId, Vec<EventId>>,
    checkpoints: HashMap<CheckpointId, CheckpointRecord>,
    checkpoint_streams: BTreeMap<(ConversationId, Option<TurnId>), Vec<CheckpointId>>,
}

impl EventStore for InMemoryEventStore {
    fn append(&mut self, event: RuntimeEvent) -> Result<StoredEvent, StorageError> {
        if self.events.contains_key(&event.id) {
            return Err(StorageError::DuplicateEvent(event.id));
        }

        let stream = self.streams.entry(event.conversation_id).or_default();
        let sequenced = assign_sequence(event, stream.len() as u64 + 1);
        let id = sequenced.id;
        let stored = StoredEvent { event: sequenced };

        stream.push(id);
        self.events.insert(id, stored.clone());
        Ok(stored)
    }

    fn list_by_conversation(
        &self,
        conversation_id: ConversationId,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        let Some(ids) = self.streams.get(&conversation_id) else {
            return Ok(Vec::new());
        };

        Ok(ids
            .iter()
            .filter_map(|id| self.events.get(id))
            .cloned()
            .collect())
    }

    fn get(&self, event_id: EventId) -> Result<Option<StoredEvent>, StorageError> {
        Ok(self.events.get(&event_id).cloned())
    }
}

impl CheckpointStore for InMemoryEventStore {
    fn put_checkpoint(&mut self, checkpoint: CheckpointRecord) -> Result<(), StorageError> {
        validate_checkpoint(&checkpoint)?;
        if self.checkpoints.contains_key(&checkpoint.checkpoint_id) {
            return Err(StorageError::DuplicateCheckpoint(checkpoint.checkpoint_id));
        }

        let key = (checkpoint.conversation_id, checkpoint.turn_id);
        let checkpoint_id = checkpoint.checkpoint_id;
        self.checkpoints.insert(checkpoint_id, checkpoint);
        self.checkpoint_streams
            .entry(key)
            .or_default()
            .push(checkpoint_id);
        Ok(())
    }

    fn latest_checkpoint(
        &self,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> Result<Option<CheckpointRecord>, StorageError> {
        let Some(ids) = self.checkpoint_streams.get(&(conversation_id, turn_id)) else {
            return Ok(None);
        };

        Ok(ids
            .iter()
            .filter_map(|id| self.checkpoints.get(id))
            .max_by_key(|checkpoint| checkpoint.event_seq)
            .cloned())
    }
}
