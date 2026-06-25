# M09 Worktree Isolation and Subagents Spec

## Purpose

Enable safe parallel work through isolated worktrees, conflict detection, reduced-capability subagents, and a team blackboard.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is Git worktree management, file locks, conflict reports, handoff envelopes, capability reduction, and shared findings.

## Scope

This milestone delivers:

- WorktreeLease contract
- ConflictReport contract
- SubagentProfile contract
- HandoffEnvelope contract
- BlackboardEntry contract

Implementation primarily touches:

- `agent-worktree`
- `agent-team`
- `agent-policy`
- `agent-storage`
- `agent-core`
- `agent-types`

## Dependencies

- M02 Tool, Policy, and Audit Kernel
- M04 Checkpoint, Recovery, and Trace Replay

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `agent-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- WorktreeLease contract
- ConflictReport contract
- SubagentProfile contract
- HandoffEnvelope contract
- BlackboardEntry contract

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

- worktree lifecycle tests
- conflict detection fixture tests
- capability reduction tests
- handoff serialization tests
- blackboard ordering tests

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- A lead agent can delegate isolated subtasks with reduced permissions.
- Subagent results return as artifacts, summaries, diffs, and trace links.
- Worktree conflicts are detected before consolidation.

## Completion Signal

M09 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
