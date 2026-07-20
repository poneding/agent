# M41-02 Sampling Model Provider Spec

## Purpose

Execute safe MCP sampling requests through an `ruse-model::ModelProvider` and return typed MCP sampling results.

## Product Context

M41-01 maps requests. M41-02 makes sampling useful in deterministic runtime tests by using `FakeModelProvider` and the existing model stream contract.

## Scope

This spec delivers:

- `McpModelSamplingProvider`
- `McpModelSamplingProvider::new`
- `McpSamplingProvider` implementation for model-backed sampling
- delta aggregation into assistant text content

Owned by later specs:

- approval UI
- transport dispatch
- tool-enabled sampling

## Architecture

`ruse-core::mcp_sampling` owns provider-backed execution. It uses `ruse_model::collect_model_event_stream` and returns `ruse_mcp::McpCreateMessageResult`.

## Public Contracts

```rust
pub struct McpModelSamplingProvider {
    pub config: McpSamplingModelBridgeConfig,
}
```

The provider stores an `Arc<dyn ModelProvider>` internally and implements `ruse_mcp::McpSamplingProvider`.

## State and Failure Handling

- Requests are validated before model streaming.
- Delta events concatenate into the response text.
- Completed finish reason becomes `stopReason`.
- Provider errors are redacted.
- Empty deltas produce an empty assistant text result.

## Security and Privacy

- Raw prompt text is not stored in provider state.
- Server hints do not select model credentials.

## Testing Strategy

- deterministic fake provider response
- empty fake provider response
- provider error mapping
- repeated calls do not retain prompt content in debug output

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m41 cargo test -p ruse-core mcp_sampling_model_provider`
- `CARGO_TARGET_DIR=target/codex-m41 cargo clippy -p ruse-core --all-targets -- -D warnings`

## Completion Signal

M41-02 is complete when an MCP sampling request can produce a deterministic MCP assistant text result through `FakeModelProvider`.
