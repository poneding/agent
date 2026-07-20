# M01-03 Storage and Replay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist `RuntimeEvent` values and replay them into deterministic event streams.

**Architecture:** `ruse-storage` owns event persistence and replay cursors. `ruse-core` consumes storage through traits and never knows whether the backend is in-memory or SQLite. M01-03 implements a sync storage contract first; later milestones can add async wrappers when scheduler and background work require them.

**Tech Stack:** Rust 2024 edition, `ruse-types`, `serde_json`, `thiserror`, `rusqlite`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Storage Dependencies

**Files:**
- Modify: `crates/ruse-storage/Cargo.toml`
- Modify: `crates/ruse-storage/src/lib.rs`

- [x] **Step 1: Add dependencies**

Add:

```toml
[dependencies]
ruse-types = { path = "../ruse-types" }
rusqlite = { version = "0.32", features = ["bundled"] }
serde_json = "1"
thiserror = "2"
```

- [x] **Step 2: Export modules**

`crates/ruse-storage/src/lib.rs` should export:

```rust
#![forbid(unsafe_code)]
//! Durable storage contracts for agent.

pub mod event_store;
pub mod memory;
pub mod replay;
pub mod sqlite;

pub use event_store::{EventStore, StorageError, StoredEvent};
pub use memory::InMemoryEventStore;
pub use replay::{ReplayCursor, ReplayEvent};
pub use sqlite::SqliteEventStore;
```

### Task 2: Define Event Store Contract

**Files:**
- Create: `crates/ruse-storage/src/event_store.rs`

- [x] **Step 1: Define storage errors and trait**

Create the public contract:

```rust
use ruse_types::{ConversationId, EventId, EventSeq, RuntimeEvent};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("duplicate event id: {0}")]
    DuplicateEvent(EventId),
    #[error("event sequence conflict for conversation")]
    SequenceConflict,
    #[error("event payload serialization failed: {0}")]
    Serialization(String),
    #[error("storage backend failed: {0}")]
    Backend(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredEvent {
    pub event: RuntimeEvent,
}

pub trait EventStore {
    fn append(&mut self, event: RuntimeEvent) -> Result<StoredEvent, StorageError>;
    fn list_by_conversation(
        &self,
        conversation_id: ConversationId,
    ) -> Result<Vec<StoredEvent>, StorageError>;
    fn get(&self, event_id: EventId) -> Result<Option<StoredEvent>, StorageError>;
}

pub(crate) fn assign_sequence(mut event: RuntimeEvent, next: u64) -> RuntimeEvent {
    event.seq = Some(EventSeq::new(next).expect("storage assigns non-zero event sequence"));
    event
}
```

- [x] **Step 2: Add contract tests**

Create tests that verify:

- appending assigns sequence `1`
- second event in the same conversation gets sequence `2`
- duplicate `EventId` is rejected
- listing by conversation returns ordered events

Run:

```bash
cargo test -p ruse-storage event_store
```

Expected: tests fail until in-memory store exists.

### Task 3: Implement In-Memory Store

**Files:**
- Create: `crates/ruse-storage/src/memory.rs`
- Create: `crates/ruse-storage/tests/event_store.rs`

- [x] **Step 1: Implement `InMemoryEventStore`**

Use vectors and maps to preserve append order:

```rust
use std::collections::{BTreeMap, HashMap};

use ruse_types::{ConversationId, EventId, RuntimeEvent};

use crate::event_store::{assign_sequence, EventStore, StorageError, StoredEvent};

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
```

- [x] **Step 2: Add shared behavior tests**

`crates/ruse-storage/tests/event_store.rs` should build fixed `RuntimeEvent` values and run the same assertions against `InMemoryEventStore`.

Run:

```bash
cargo test -p ruse-storage in_memory_assigns_ordered_sequences
```

Expected: test passes.

### Task 4: Implement SQLite Store

**Files:**
- Create: `crates/ruse-storage/src/sqlite.rs`
- Modify: `crates/ruse-storage/tests/event_store.rs`

- [x] **Step 1: Create SQLite schema on open**

`SqliteEventStore::open_in_memory()` should create:

```sql
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
```

- [x] **Step 2: Implement append and list**

SQLite append must:

- reject duplicate `event_id`
- assign `seq = max(seq) + 1` inside a transaction
- serialize the full `RuntimeEvent` after sequence assignment
- return the stored event

- [x] **Step 3: Run SQLite tests**

Run:

```bash
cargo test -p ruse-storage sqlite_assigns_ordered_sequences
cargo test -p ruse-storage sqlite_rejects_duplicate_event_id
```

Expected: SQLite behavior matches in-memory behavior.

### Task 5: Implement Replay Cursor

**Files:**
- Create: `crates/ruse-storage/src/replay.rs`
- Modify: `crates/ruse-storage/tests/event_store.rs`

- [x] **Step 1: Define replay types**

Create:

```rust
use ruse_types::{ConversationId, EventSeq, RuntimeEvent};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayEvent {
    pub seq: EventSeq,
    pub event: RuntimeEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCursor {
    pub conversation_id: ConversationId,
    pub next_seq: Option<EventSeq>,
}
```

- [x] **Step 2: Add replay tests**

Tests must prove:

- replay returns events in sequence order
- replay preserves `schema_version`
- replay over an empty conversation returns an empty stream

Run:

```bash
cargo test -p ruse-storage replay_returns_ordered_events
cargo test -p ruse-storage replay_from_sequence_skips_prior_events
```

Expected: replay tests pass for both in-memory and SQLite stores.

### Task 6: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run standard checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands pass.

- [x] **Step 2: Advance active focus**

Update active focus from `M01-03-storage-replay` to `M01-04-fake-model`.

- [x] **Step 3: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m01-03 storage replay"
```

Expected: commit succeeds.
