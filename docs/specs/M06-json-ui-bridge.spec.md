# M06 JSON Mode and UI Bridge Spec

## Purpose

Expose the shared runtime to CLI JSON mode and future Desktop clients through stable command and event schemas.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is UI-facing events, UI commands, JSON event stream mode, and bridge snapshot tests.

## Scope

This milestone delivers:

- UiEvent contract
- UiCommand contract
- JSON stream framing contract
- Runtime-to-UI normalization contract

Implementation primarily touches:

- `ruse-ui-bridge`
- `ruse-cli`
- `ruse-core`
- `ruse-types`

## Dependencies

- M01 Runtime Spine MVP
- M03 Local Coding Loop

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `ruse-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- UiEvent contract
- UiCommand contract
- JSON stream framing contract
- Runtime-to-UI normalization contract

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

- UI event JSON snapshots
- UI command deserialization tests
- CLI JSON stream tests
- bridge normalization tests

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- Interactive CLI and JSON mode observe the same runtime events.
- A UI client can subscribe to progress without linking directly to ruse-core.

## Completion Signal

M06 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
