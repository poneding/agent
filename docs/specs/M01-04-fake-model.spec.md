# M01-04 Fake Model Provider Spec

## Purpose

Provide a scripted streaming model provider for deterministic runtime tests.

## Product Context

This slice is part of M01 Runtime Spine MVP. Its design center is ModelProvider trait, scripted deltas, finish reasons, request metadata, and fake model fixtures. It builds on earlier M01 slices and keeps the runtime spine testable before tool, policy, memory, scheduler, or UI bridge capabilities are introduced.

## Scope

This slice delivers:

- scripted text deltas stream in order
- finish reason is emitted
- empty script returns stop event
- request metadata is preserved
- fake provider is Send + Sync

Primary implementation files:

- `crates/agent-model/src/lib.rs`
- `crates/agent-model/src/provider.rs`
- `crates/agent-model/src/request.rs`
- `crates/agent-model/src/stream.rs`
- `crates/agent-model/src/fake.rs`
- `crates/agent-model/tests/fake_model.rs`

## Boundaries

- The slice may depend on earlier M01 contracts.
- The slice does not add M02 tool or permission behavior.
- The slice does not add M03 interactive CLI behavior beyond smoke coverage when explicitly named.
- Durable schemas introduced here must remain compatible with M01 replay requirements.

## Architecture

The implementation stays within the runtime spine crates and keeps behavior deterministic. Public contracts must be testable with local fixtures and without a live model provider or network access.

## Testing Strategy

Required tests:

- scripted text deltas stream in order
- finish reason is emitted
- empty script returns stop event
- request metadata is preserved
- fake provider is Send + Sync

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- The slice's public API is implemented in the files listed above.
- Focused tests for M01-04 pass.
- Workspace tests pass.
- The next M01 slice can consume this slice without changing its public contract.

## Completion Signal

M01-04 is complete when its implementation, tests, and plan checklist are complete and the active focus can advance to the next M01 slice.
