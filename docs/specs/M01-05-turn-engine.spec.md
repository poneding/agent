# M01-05 Turn Engine Spec

## Purpose

Implement a minimal turn state machine that consumes user input, streams fake model output, emits runtime events, and reaches a final response.

## Product Context

This slice is part of M01 Runtime Spine MVP. Its design center is TurnEngine, turn input, minimal context assembly, model stream consumption, state changes, event emission, and final response construction. It builds on earlier M01 slices and keeps the runtime spine testable before tool, policy, memory, scheduler, or UI bridge capabilities are introduced.

## Scope

This slice delivers:

- turn starts from accepted input
- state changes emit ordered events
- model deltas aggregate into final response
- model error produces TurnFailed
- completed turn reaches Done

Primary implementation files:

- `crates/agent-core/src/lib.rs`
- `crates/agent-core/src/turn.rs`
- `crates/agent-core/src/runtime.rs`
- `crates/agent-core/src/context.rs`
- `crates/agent-core/tests/turn_engine.rs`

## Boundaries

- The slice may depend on earlier M01 contracts.
- The slice does not add M02 tool or permission behavior.
- The slice does not add M03 interactive CLI behavior beyond smoke coverage when explicitly named.
- Durable schemas introduced here must remain compatible with M01 replay requirements.

## Architecture

The implementation stays within the runtime spine crates and keeps behavior deterministic. Public contracts must be testable with local fixtures and without a live model provider or network access.

## Testing Strategy

Required tests:

- turn starts from accepted input
- state changes emit ordered events
- model deltas aggregate into final response
- model error produces TurnFailed
- completed turn reaches Done

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- The slice's public API is implemented in the files listed above.
- Focused tests for M01-05 pass.
- Workspace tests pass.
- The next M01 slice can consume this slice without changing its public contract.

## Completion Signal

M01-05 is complete when its implementation, tests, and plan checklist are complete and the active focus can advance to the next M01 slice.
