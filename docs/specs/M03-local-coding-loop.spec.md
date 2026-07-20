# M03 Local Coding Loop Spec

## Purpose

Make the runtime useful in a local repository through an interactive CLI coding loop.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is TTY interaction, streaming output, file edits, shell commands, Git inspection, and approval prompts.

## Scope

This milestone delivers:

- CLI command contract
- File edit diff contract
- Shell execution contract
- Git status/diff contract
- Approval prompt contract

Implementation primarily touches:

- `ruse-cli`
- `ruse-tool`
- `ruse-policy`
- `ruse-core`
- `ruse-types`

## Dependencies

- M01 Runtime Spine MVP
- M02 Tool, Policy, and Audit Kernel

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `ruse-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- CLI command contract
- File edit diff contract
- Shell execution contract
- Git status/diff contract
- Approval prompt contract

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

- CLI smoke and snapshot tests
- file edit diff tests
- shell timeout tests
- git status fixture tests
- local coding E2E test

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- A user can inspect a repository, read files, propose edits, and run approved commands.
- Tool calls remain visible, permissioned, persisted, and replayable.
- CLI reports final answer, changed files, command results, and audit summary.

## Completion Signal

M03 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
