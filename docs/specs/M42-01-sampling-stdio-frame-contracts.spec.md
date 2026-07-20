# M42-01 Sampling Stdio Frame Contracts Spec

## Purpose

Add stable MCP sampling stdio frame contracts for `sampling/createMessage` requests and JSON-RPC response rendering.

## Product Context

M42 needs a typed protocol surface before adding dispatcher behavior. M42-01 creates the constants and helpers that later slices execute.

## Scope

This spec delivers:

- `MCP_SAMPLING_CREATE_MESSAGE_METHOD`
- sampling request frame parsing helpers
- success and failure response rendering helpers
- focused serialization tests

Owned by later M42 specs:

- invoking `McpSamplingProvider`
- validation-before-provider behavior
- redaction hardening

## Architecture

Contracts live in `ruse-mcp`, likely in a new `client_sampling_stdio` module or equivalent. They reuse existing JSON-RPC envelope types from `server.rs` rather than introducing a second envelope schema.

## Public Contracts

```rust
pub const MCP_SAMPLING_CREATE_MESSAGE_METHOD: &str = "sampling/createMessage";

pub fn parse_mcp_sampling_stdio_request_frame(
    frame: &str,
) -> Result<Option<(McpJsonRpcId, McpCreateMessageRequestParams)>, McpJsonRpcError>;
```

Notifications without `id` return `Ok(None)` because JSON-RPC notifications do not produce response frames.

## State and Failure Handling

- Malformed JSON returns parse error.
- Non-`2.0` frames return invalid request.
- Unsupported methods return method not found.
- Invalid params return invalid params.
- Method names and diagnostics are bounded and redacted.

## Security and Privacy

- Parser errors must not include raw `params`, prompt text, system prompt text, or secret-like method names.

## Testing Strategy

- valid request frame decodes to sampling params
- notification frame returns no response requirement
- unsupported method returns redacted method-not-found error
- malformed and invalid params produce stable JSON-RPC errors

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m42 cargo test -p ruse-mcp mcp_sampling_stdio_contracts`

## Completion Signal

M42-01 is complete when sampling stdio frame contracts are public, tested, and exported from `ruse-mcp`.
