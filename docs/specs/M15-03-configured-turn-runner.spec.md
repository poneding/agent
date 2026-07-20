# M15-03 Configured Turn Runner Spec

## Purpose

Run a full `TurnEngine` turn using the provider/model selected by effective runtime config.

## Product Context

The existing `TurnEngine` can run with a provider passed directly by tests. M15-03 creates the runtime-facing entry point that accepts effective config and registry inputs.

## Scope

This spec delivers:

- `ConfiguredTurnInput`
- `ConfiguredTurnOutcome`
- `ConfiguredTurnRunner`
- integration with `ModelRuntimeResolver`
- integration with `ModelInvocationRunner`

Owned by later specs:

- full autonomous tool loop recursion
- Desktop prompt execution
- live HTTP transport construction

## Architecture

`ConfiguredTurnRunner` lives in `ruse-core::model_runtime` and wraps `TurnEngine`. It accepts `RuntimeIds`, user message, `EffectiveRuntimeConfig`, provider registry, and requirements. It returns final message, emitted events, selected provider/model, and invocation attempt history.

## Public Contracts

```rust
pub struct ConfiguredTurnInput {
    pub ids: RuntimeIds,
    pub user_message: String,
    pub effective_config: EffectiveRuntimeConfig,
    pub requirements: ModelRequestRequirements,
}

pub struct ConfiguredTurnOutcome {
    pub final_message: String,
    pub selected_provider: String,
    pub selected_model: String,
    pub events: Vec<RuntimeEvent>,
    pub attempts: Vec<ModelInvocationAttempt>,
}
```

## State And Failure Handling

- empty user message is rejected before model invocation
- provider resolution failure returns a typed error and no transport calls
- model failure returns failed turn events from `TurnEngine`
- successful turns reach `TurnState::Done`

## Security And Privacy

- outcome events follow existing runtime-event redaction rules
- selected provider/model can be surfaced to CLI/Desktop
- raw config files and secrets are never included in outcome errors

## Testing Strategy

- configured turn uses selected provider/model from effective config
- empty user message is rejected
- resolver failure prevents model invocation
- successful fake turn emits final response and done state

## Acceptance Criteria

- Core has one reusable API for config-driven prompt turns.
- CLI can call configured turn execution without touching provider internals.

## Completion Signal

M15-03 is complete when `cargo test -p ruse-core --test configured_turn_runner` and workspace gates pass, status advances to M15-04, and the slice is committed.
