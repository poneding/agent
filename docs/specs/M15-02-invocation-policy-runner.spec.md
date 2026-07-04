# M15-02 Invocation Policy Runner Spec

## Purpose

Apply configured retry and fallback policy around model invocation without embedding policy logic inside `TurnEngine`.

## Product Context

M13 created `ModelInvocationPolicy`; M15-02 makes it operational. The runtime should retry retryable provider errors and fall back to configured providers deterministically.

## Scope

This spec delivers:

- `ModelInvocationRunner`
- `ModelInvocationAttempt`
- `ModelInvocationOutcome`
- retryable failure classification
- fallback provider selection through `ModelProviderRegistry`

Owned by later specs:

- durable invocation attempt events
- cost/accounting budgets
- provider health tracking

## Architecture

The runner lives in `agent-core::model_runtime`. It receives a selected runtime, a registry, request requirements, and a request template. It calls providers through the `ModelProvider` trait and returns a `ModelStreamResult`.

## Public Contracts

```rust
pub struct ModelInvocationAttempt {
    pub provider_id: String,
    pub model: String,
    pub attempt_index: u32,
}

pub struct ModelInvocationOutcome {
    pub provider_id: String,
    pub model: String,
    pub attempts: Vec<ModelInvocationAttempt>,
    pub stream: ModelStreamResult,
}
```

## State And Failure Handling

- first attempt uses selected provider/model
- retry attempts reuse the same provider/model
- fallback attempts use fallback provider default models unless explicit model override is present
- non-retryable errors stop immediately
- exhausted retry/fallback returns a typed error with attempt history

## Security And Privacy

- attempt history contains provider/model ids only
- model request messages are not copied into errors
- provider errors are used through their redacted `Display` output

## Testing Strategy

- retryable failure retries up to `max_attempts`
- non-retryable failure does not retry
- fallback provider is used after selected provider fails
- missing fallback provider records safe typed error

## Acceptance Criteria

- Invocation policy behavior is deterministic and independent from CLI.
- Later turn execution can call one runner instead of hand-rolling retry/fallback.

## Completion Signal

M15-02 is complete when `cargo test -p agent-core --test model_invocation_policy_runner` and workspace gates pass, status advances to M15-03, and the slice is committed.
