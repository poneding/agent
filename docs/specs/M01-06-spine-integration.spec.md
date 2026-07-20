# M01-06 Spine Integration Spec

## Purpose

Run an end-to-end fake turn, persist emitted events, reload them, and verify the replayed final state.

## Product Context

This slice is part of M01 Runtime Spine MVP. Its design center is ruse-core plus ruse-model plus ruse-storage integration, persisted fake turn fixture, replayed final response, and CLI smoke validation. It builds on earlier M01 slices and keeps the runtime spine testable before tool, policy, memory, scheduler, or UI bridge capabilities are introduced.

## Scope

This slice delivers:

- end-to-end fake turn persists events
- replay reconstructs final response
- event sequence is contiguous
- CLI smoke can invoke runtime banner
- workspace tests pass together

Primary implementation files:

- `crates/ruse-core/tests/spine_integration.rs`
- `crates/ruse-cli/tests/runtime_spine.rs`
- `crates/ruse-storage/tests/replay_integration.rs`

## Boundaries

- The slice may depend on earlier M01 contracts.
- The slice does not add M02 tool or permission behavior.
- The slice does not add M03 interactive CLI behavior beyond smoke coverage when explicitly named.
- Durable schemas introduced here must remain compatible with M01 replay requirements.

## Architecture

The implementation stays within the runtime spine crates and keeps behavior deterministic. Public contracts must be testable with local fixtures and without a live model provider or network access.

## Testing Strategy

Required tests:

- end-to-end fake turn persists events
- replay reconstructs final response
- event sequence is contiguous
- CLI smoke can invoke runtime banner
- workspace tests pass together

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- The slice's public API is implemented in the files listed above.
- Focused tests for M01-06 pass.
- Workspace tests pass.
- The next M01 slice can consume this slice without changing its public contract.

## Completion Signal

M01-06 is complete when its implementation, tests, and plan checklist are complete and the active focus can advance to the next M01 slice.
