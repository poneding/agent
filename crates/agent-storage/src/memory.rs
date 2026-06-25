use std::collections::{BTreeMap, HashMap};

use agent_types::{ConversationId, EventId, RuntimeEvent};

use crate::event_store::{EventStore, StorageError, StoredEvent, assign_sequence};

#[derive(Default)]
pub struct InMemoryEventStore {
    events: HashMap<EventId, StoredEvent>,
    streams: BTreeMap<ConversationId, Vec<EventId>>,
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
