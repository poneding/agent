use agent_types::{ConversationId, EventId, RuntimeEvent};
use rusqlite::{Connection, ErrorCode, OptionalExtension, Transaction, params};

use crate::event_store::{EventStore, StorageError, StoredEvent, assign_sequence};

pub struct SqliteEventStore {
    conn: Connection,
}

impl SqliteEventStore {
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory().map_err(backend)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), StorageError> {
        self.conn
            .execute_batch(
                r"
                CREATE TABLE IF NOT EXISTS runtime_events (
                  event_id TEXT PRIMARY KEY,
                  workspace_id TEXT NOT NULL,
                  conversation_id TEXT NOT NULL,
                  seq INTEGER NOT NULL,
                  occurred_at TEXT NOT NULL,
                  schema_version INTEGER NOT NULL,
                  payload_json TEXT NOT NULL,
                  UNIQUE(conversation_id, seq)
                );

                CREATE INDEX IF NOT EXISTS idx_runtime_events_conversation_seq
                ON runtime_events(conversation_id, seq);
                ",
            )
            .map_err(backend)
    }

    fn serialize_event(event: &RuntimeEvent) -> Result<(String, String), StorageError> {
        let value = serde_json::to_value(event)
            .map_err(|error| StorageError::Serialization(error.to_string()))?;
        let occurred_at = value["occurred_at"]
            .as_str()
            .ok_or_else(|| StorageError::Serialization("occurred_at must be a string".to_string()))?
            .to_string();
        let payload_json = serde_json::to_string(event)
            .map_err(|error| StorageError::Serialization(error.to_string()))?;
        Ok((occurred_at, payload_json))
    }

    fn event_exists(tx: &Transaction<'_>, event_id: EventId) -> Result<bool, StorageError> {
        let exists = tx
            .query_row(
                "SELECT 1 FROM runtime_events WHERE event_id = ?1",
                params![event_id.to_string()],
                |_| Ok(()),
            )
            .optional()
            .map_err(backend)?;

        Ok(exists.is_some())
    }

    fn next_sequence(
        tx: &Transaction<'_>,
        conversation_id: ConversationId,
    ) -> Result<u64, StorageError> {
        tx.query_row(
            "SELECT COALESCE(MAX(seq), 0) + 1 FROM runtime_events WHERE conversation_id = ?1",
            params![conversation_id.to_string()],
            |row| row.get::<_, u64>(0),
        )
        .map_err(backend)
    }
}

impl EventStore for SqliteEventStore {
    fn append(&mut self, event: RuntimeEvent) -> Result<StoredEvent, StorageError> {
        let tx = self.conn.transaction().map_err(backend)?;

        if Self::event_exists(&tx, event.id)? {
            return Err(StorageError::DuplicateEvent(event.id));
        }

        let seq = Self::next_sequence(&tx, event.conversation_id)?;
        let sequenced = assign_sequence(event, seq);
        let (occurred_at, payload_json) = Self::serialize_event(&sequenced)?;

        tx.execute(
            r"
            INSERT INTO runtime_events (
                event_id,
                workspace_id,
                conversation_id,
                seq,
                occurred_at,
                schema_version,
                payload_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                sequenced.id.to_string(),
                sequenced.workspace_id.to_string(),
                sequenced.conversation_id.to_string(),
                sequenced.seq.expect("assigned sequence").get(),
                occurred_at,
                sequenced.schema_version.get(),
                payload_json
            ],
        )
        .map_err(|error| insert_error(error, sequenced.id))?;
        tx.commit().map_err(backend)?;

        Ok(StoredEvent { event: sequenced })
    }

    fn list_by_conversation(
        &self,
        conversation_id: ConversationId,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        let mut stmt = self
            .conn
            .prepare(
                r"
                SELECT payload_json
                FROM runtime_events
                WHERE conversation_id = ?1
                ORDER BY seq ASC
                ",
            )
            .map_err(backend)?;
        let mut rows = stmt
            .query(params![conversation_id.to_string()])
            .map_err(backend)?;
        let mut events = Vec::new();

        while let Some(row) = rows.next().map_err(backend)? {
            let payload_json: String = row.get(0).map_err(backend)?;
            let event = serde_json::from_str(&payload_json)
                .map_err(|error| StorageError::Serialization(error.to_string()))?;
            events.push(StoredEvent { event });
        }

        Ok(events)
    }

    fn get(&self, event_id: EventId) -> Result<Option<StoredEvent>, StorageError> {
        let payload_json = self
            .conn
            .query_row(
                "SELECT payload_json FROM runtime_events WHERE event_id = ?1",
                params![event_id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(backend)?;

        payload_json
            .map(|json| {
                serde_json::from_str(&json)
                    .map(|event| StoredEvent { event })
                    .map_err(|error| StorageError::Serialization(error.to_string()))
            })
            .transpose()
    }
}

fn backend(error: rusqlite::Error) -> StorageError {
    StorageError::Backend(error.to_string())
}

fn insert_error(error: rusqlite::Error, event_id: EventId) -> StorageError {
    if let rusqlite::Error::SqliteFailure(failure, Some(message)) = &error {
        if failure.code == ErrorCode::ConstraintViolation {
            if message.contains("runtime_events.event_id") {
                return StorageError::DuplicateEvent(event_id);
            }
            if message.contains("runtime_events.conversation_id")
                || message.contains("runtime_events.seq")
            {
                return StorageError::SequenceConflict;
            }
        }
    }

    backend(error)
}
