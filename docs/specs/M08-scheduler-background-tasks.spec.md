# M08 Background Tasks and Scheduler Spec

## Purpose

Support durable long-running and recurring work with leases, retries, quotas, and notifications.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is task state machine, durable queue, leases, cancellation, pause/resume, retries, quotas, and notification events.

## Scope

This milestone delivers:

- TaskRun contract
- TaskLease contract
- RetryPolicy contract
- QuotaPolicy contract
- TaskNotification contract

Implementation is split across:

- `M08-01-task-state-machine`
- `M08-02-queue-leases`
- `M08-03-quotas`
- `M08-04-notifications`

Implementation primarily touches:

- `agent-scheduler`
- `agent-storage`
- `agent-core`
- `agent-types`

## Dependencies

- M01 Runtime Spine MVP
- M04 Checkpoint, Recovery, and Trace Replay

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `agent-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- TaskRun contract
- TaskLease contract
- RetryPolicy contract
- QuotaPolicy contract
- TaskNotification contract

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

- task transition tests
- lease acquisition tests
- retry schedule tests
- quota rejection tests
- waiting-approval notification tests

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- A background task can run, pause for approval, resume, complete, and expose a trace.
- Concurrent tasks respect workspace and system limits.

## Completion Signal

M08 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
