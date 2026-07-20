# M17-01 Model Stream Trait Spec

## Purpose

Replace the collected `ModelStreamResult = Vec<ModelStreamEvent>` provider contract with a true async event stream so model deltas can be consumed as they arrive.

## Product Context

M15 configured turns still wait for a complete vector before CLI or UI can render anything. M17 needs a provider-neutral stream boundary before live HTTP, CLI streaming, cancellation, and replay can be implemented.

## Scope

This slice delivers:

- `ModelEventStream` as the provider-neutral async stream type
- `ModelProvider::stream` returning a stream instead of a collected vector
- fixture helpers for collecting streams in deterministic tests
- fake provider migration to stream events incrementally
- core model invocation migration that preserves existing final outcomes

Out of scope:

- real HTTP transport
- CLI streaming rendering
- cancellation policy beyond stream drop semantics

## Architecture

`ruse-model` owns the stream type and provider trait. `ruse-core` consumes streams but remains provider-neutral. Existing tests that need a complete result use an explicit collection helper so collecting is a test/runtime choice, not the provider contract.

Target contract:

```rust
pub type ModelEventStream =
    Pin<Box<dyn Stream<Item = Result<ModelStreamEvent, ModelError>> + Send>>;

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, ModelError>;
}
```

`ModelStreamEvent` remains the stable vocabulary for M17:

- `Started`
- `Delta`
- `ToolCall`
- `Completed`

M18 can add partial tool-call accumulation without changing this slice's provider boundary.

## State And Failure Handling

- Provider setup errors return before a stream is yielded.
- Stream item errors represent failures after streaming starts.
- Test collection helpers stop on the first stream item error.
- Core converts stream item errors into the same model failure path used today.

## Security And Privacy

The stream type carries provider-neutral events only. Provider-specific raw SSE frames, authorization headers, secret refs, and raw error bodies stay inside provider implementations.

## Testing Strategy

- fake providers yield a `ModelEventStream`
- stream collection preserves event order
- stream item errors propagate as `ModelError`
- configured turn execution still emits final responses from fake streams
- existing provider registry capability tests remain unchanged

## Acceptance Criteria

- No public provider trait returns `Vec<ModelStreamEvent>`.
- Existing deterministic fake turns still pass.
- Stream collection is explicit and tested.
- No live network is required.

## Completion Signal

M17-01 is complete when focused `ruse-model` and `ruse-core` stream migration tests pass and `cargo xtask check` is green.
