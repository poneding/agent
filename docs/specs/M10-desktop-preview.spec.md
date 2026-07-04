# M10 Desktop Preview Spec

## Purpose

Provide a visual Desktop client over the shared runtime bridge.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is desktop shell, conversation view, task list, permission dialog, diff review, trace timeline, and settings surfaces.

## Scope

This milestone delivers:

- DesktopSession contract
- PermissionDialog contract
- DiffReview contract
- TraceTimeline contract
- Settings contract

Implementation primarily touches:

- `agent-desktop`
- `agent-ui-bridge`
- `agent-core`
- `agent-types`

## Dependencies

- M06 JSON Mode and UI Bridge
- M08 Background Tasks and Scheduler

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `agent-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- DesktopSession contract
- PermissionDialog contract
- DiffReview contract
- TraceTimeline contract
- Settings contract

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

- bridge contract tests
- desktop command fixtures
- permission dialog state tests
- trace timeline snapshot tests

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- Desktop can run the same local coding loop as CLI.
- Permission decisions, traces, diffs, and tasks reflect shared runtime state.

## Completion Signal

M10 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
