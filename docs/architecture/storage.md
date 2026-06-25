# Storage

Storage makes runtime behavior durable without leaking database details into core orchestration.

## Storage Backends

The local-first backend is SQLite. Traits keep storage replaceable for tests and future server deployments.

Primary stores:

- event store
- checkpoint store
- artifact store
- config store
- memory store
- task store
- audit store

## Event Store

The event store owns sequence assignment.

Required behavior:

- append events atomically
- assign monotonic per-conversation sequence numbers
- read streams by conversation
- read ranges by sequence
- reject duplicate event IDs
- preserve schema version and payload JSON

## Checkpoint Store

Checkpoints reference event positions.

Required behavior:

- save checkpoint with conversation, turn/task, and last event sequence
- load latest valid checkpoint
- reject checkpoint if referenced event range is unavailable
- support checkpoint cleanup by retention policy

## Artifact Store

Large outputs stay out of prompt context and event payloads.

Artifact records include:

- content hash
- media type
- size
- path or blob reference
- origin event ID
- retention policy

## SQLite Rules

- Use migrations for schema changes.
- Keep SQL in `agent-storage`.
- Use transactions for multi-row updates.
- Store durable payloads as JSON where schema evolution is expected.
- Index event stream lookups by workspace, conversation, turn, task, and sequence.

## Testing Requirements

- In-memory and SQLite stores pass the same behavior tests.
- Migration fixtures cover every schema version.
- Event append/read tests prove order.
- Artifact tests prove hash and size validation.
- Recovery tests prove checkpoint plus event stream consistency.
