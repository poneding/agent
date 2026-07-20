# M01 Runtime Spine MVP Spec

## Purpose

Create the smallest event-sourced Rust runtime that can execute and replay a deterministic agent turn.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is typed runtime events, storage, fake model streaming, turn orchestration, and deterministic replay.

## Scope

This milestone delivers:

- RuntimeEvent envelope
- EventStore contract
- ModelProvider fake stream contract
- TurnEngine lifecycle contract
- Replay consistency contract

Implementation primarily touches:

- `ruse-types`
- `ruse-storage`
- `ruse-model`
- `ruse-core`
- `ruse-cli`

## Dependencies

- Foundation documents
- M01-01 workspace scaffold

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `ruse-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- RuntimeEvent envelope
- EventStore contract
- ModelProvider fake stream contract
- TurnEngine lifecycle contract
- Replay consistency contract

Contract rules:

- Public data crossing crate boundaries uses explicit Rust types.
- Durable data includes a schema or compatibility strategy.
- Side-effecting behavior emits runtime events or audit records.
- UI-facing behavior is expressed through events or bridge contracts rather than direct UI calls.

## State and Failure Handling

- State transitions are explicit and test-covered.
- Recoverable failures become typed runtime errors with stable codes.
- Non-recoverable failures stop the current task or turn with persisted diagnostic context.
- Retried work must be idempotent or guarded by a recorded lease, checkpoint, or correlation identifier.

## Security and Privacy

- Capabilities are explicit at crate and runtime boundaries.
- Secrets are never persisted in plain runtime events, snapshots, trace bundles, or test fixtures.
- External input from models, repositories, plugins, MCP servers, tools, and UI clients is treated as untrusted.
- Audit-relevant decisions include actor, scope, resource, action, decision, and reason.

## Testing Strategy

Required test coverage:

- event JSON round trips
- event store append/read ordering
- fake model scripted stream
- turn engine final state
- persisted fake turn replay

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- A fake turn runs from user input to final response.
- Every meaningful state transition emits a typed event.
- Persisted events replay into the same final state.

## Completion Signal

M01 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
