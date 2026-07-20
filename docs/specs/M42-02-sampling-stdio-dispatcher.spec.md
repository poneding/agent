# M42-02 Sampling Stdio Dispatcher Spec

## Purpose

Invoke an `McpSamplingProvider` from a valid sampling stdio request frame and render a JSON-RPC success frame.

## Product Context

M42-01 makes sampling frames parseable. M42-02 makes the path useful by dispatching valid requests through the same safe sampling provider helper used by M40 and M41.

## Scope

This spec delivers:

- `handle_mcp_sampling_stdio_frame`
- success response rendering for `McpCreateMessageResult`
- disabled config failure handling
- fixture provider integration tests

Owned by later M42 specs:

- validation-specific call count tests
- redaction-specific diagnostics
- README hardening docs

## Architecture

`ruse-mcp` dispatcher calls `create_mcp_sampling_message(provider, config, request).await` or a synchronous fixture equivalent depending on final async shape. The public API should make async execution explicit if provider invocation is async.

## Public Contracts

```rust
pub async fn handle_mcp_sampling_stdio_frame(
    provider: &dyn McpSamplingProvider,
    config: &McpServerConfig,
    frame: &str,
) -> String;
```

If the implementation keeps parsing and rendering separate, the public dispatcher can be named differently but must be exported and documented.

## State and Failure Handling

- Successful responses preserve the original JSON-RPC `id`.
- Successful responses serialize the MCP sampling result under `result`.
- Disabled configs return JSON-RPC failure and do not call the provider.
- Notifications return an empty string.

## Security and Privacy

- Success frames necessarily include model response content because that is the MCP sampling result; public diagnostics and errors remain redacted.

## Testing Strategy

- valid frame returns assistant text result
- numeric and string ids round-trip
- notifications produce no response
- disabled config rejects before provider call

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m42 cargo test -p ruse-mcp mcp_sampling_stdio_dispatch`

## Completion Signal

M42-02 is complete when a fixture sampling provider can turn a valid JSON-RPC request frame into a valid JSON-RPC success frame.
