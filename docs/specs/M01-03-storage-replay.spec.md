# M01-03 Storage and Replay Spec

## Purpose

Persist RuntimeEvent values and replay them into deterministic event streams.

## Product Context

This slice is part of M01 Runtime Spine MVP. Its design center is EventStore trait, in-memory store, SQLite store, append ordering, stream loading, and replay cursor semantics. It builds on earlier M01 slices and keeps the runtime spine testable before tool, policy, memory, scheduler, or UI bridge capabilities are introduced.

## Scope

This slice delivers:

- in-memory append/read ordering
- SQLite append/read ordering
- duplicate EventId rejection
- conversation/EventSeq uniqueness protection
- conversation stream replay
- schema_version preservation

Primary implementation files:

- `crates/agent-storage/src/lib.rs`
- `crates/agent-storage/src/event_store.rs`
- `crates/agent-storage/src/memory.rs`
- `crates/agent-storage/src/sqlite.rs`
- `crates/agent-storage/src/replay.rs`
- `crates/agent-storage/tests/event_store.rs`

## Boundaries

- The slice may depend on earlier M01 contracts.
- The slice does not add M02 tool or permission behavior.
- The slice does not add M03 interactive CLI behavior beyond smoke coverage when explicitly named.
- Durable schemas introduced here must remain compatible with M01 replay requirements.

## Architecture

The implementation stays within the runtime spine crates and keeps behavior deterministic. Public contracts must be testable with local fixtures and without a live model provider or network access.

## Testing Strategy

Required tests:

- in-memory append/read ordering
- SQLite append/read ordering
- duplicate EventId rejection
- conversation/EventSeq uniqueness protection
- conversation stream replay
- schema_version preservation

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- The slice's public API is implemented in the files listed above.
- Focused tests for M01-03 pass.
- Workspace tests pass.
- The next M01 slice can consume this slice without changing its public contract.

## Completion Signal

M01-03 is complete when its implementation, tests, and plan checklist are complete and the active focus can advance to the next M01 slice.
