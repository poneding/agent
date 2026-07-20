# M17-05 Streaming Retry, Cancellation, and Replay Spec

## Purpose

Make streaming model execution retryable, cancellable, and replayable without duplicating completed side effects.

## Product Context

Live streams can fail before or after partial output. The runtime needs clear policy boundaries so retry/fallback behavior remains auditable and future M18 tool loops do not accidentally replay side effects.

## Scope

This slice delivers:

- retry/fallback over stream setup and pre-side-effect stream failures
- cancellation state for model streams
- runtime events that distinguish failed, cancelled, retried, and completed attempts
- replay tests proving streamed deltas reconstruct final output
- guardrails for failures after tool calls are introduced later

Out of scope:

- tool execution cancellation
- approval resume UX
- background task cancellation UI

## Architecture

`ruse-core::ModelInvocationRunner` owns policy. Providers emit streams and classify errors; core decides whether to retry or fall back. Runtime events record attempts, deltas, completion, failure, and cancellation in order. Existing event sourcing remains the replay source of truth.

## State And Failure Handling

- setup failures are retryable when classified retryable by policy
- stream item errors before side-effecting tool observations can retry if policy allows
- cancellation closes/drops the stream and emits cancelled runtime state
- retries preserve attempt indices and provider/model ids
- replay reconstructs final response only from persisted deltas/final events

## Security And Privacy

Error messages remain redacted. Cancellation and retry diagnostics must not include raw request bodies, raw SSE frames, authorization headers, or secret refs beyond safe labels.

## Testing Strategy

- retryable stream setup error retries according to policy
- non-retryable auth error does not retry
- fallback provider is used after selected provider stream failure
- cancellation emits cancelled state and stops consuming stream
- replay reconstructs streamed final response from events

## Acceptance Criteria

- Streaming failures obey existing invocation policy.
- Cancellation is observable and deterministic.
- Replay remains event-sourced and does not require provider access.

## Completion Signal

M17-05 is complete when streaming retry/cancel/replay tests pass, `cargo xtask check` is green, and M17 can be marked complete.
