# M04 Checkpoint, Recovery, and Trace Replay Spec

## Purpose

Make interrupted turns durable, inspectable, and replayable without duplicating completed side effects.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is checkpoint records, idempotency keys, crash recovery, trace commands, and compatibility replay tests.

## Scope

This milestone delivers:

- CheckpointStore contract
- IdempotencyKey contract
- RecoveryPlan contract
- TraceQuery contract
- Replay compatibility contract

Implementation primarily touches:

- `agent-storage`
- `agent-core`
- `agent-cli`
- `agent-types`

## Dependencies

- M01 Runtime Spine MVP
- M02 Tool, Policy, and Audit Kernel
- M03 Local Coding Loop

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `agent-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- CheckpointStore contract
- IdempotencyKey contract
- RecoveryPlan contract
- TraceQuery contract
- Replay compatibility contract

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

- checkpoint serialization tests
- idempotent tool replay tests
- interrupted turn recovery test
- trace command snapshot tests
- old trace replay fixture tests

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- Interrupted turns resume from checkpoint plus event log.
- Completed side-effecting calls are not duplicated.
- Users can inspect model, tool, policy, and final-state traces.

## Completion Signal

M04 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
