# M02 Tool, Policy, and Audit Kernel Spec

## Purpose

Add controlled tool execution through one registry, one permission path, and one audit trail.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is tool traits, schema validation, capability decisions, leases, read-only built-ins, and audit events.

## Scope

This milestone delivers:

- Tool trait
- ToolRegistry lookup contract
- PermissionRequest and PolicyDecision contract
- PermissionLease contract
- Tool audit event contract

Implementation primarily touches:

- `ruse-tool`
- `ruse-policy`
- `ruse-sandbox`
- `ruse-core`
- `ruse-types`

## Dependencies

- M01 Runtime Spine MVP

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `ruse-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- Tool trait
- ToolRegistry lookup contract
- PermissionRequest and PolicyDecision contract
- PermissionLease contract
- Tool audit event contract

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

- tool schema validation
- registry duplicate rejection
- allow decision execution
- deny decision short circuit
- audit event ordering

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- A fake model can propose a tool call.
- Runtime validates, authorizes, executes, records, and observes the call.
- Denied and approval-required calls become deterministic runtime states.

## Completion Signal

M02 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
