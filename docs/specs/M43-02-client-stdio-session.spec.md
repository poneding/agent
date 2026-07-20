# M43-02 Client Stdio Session Spec

## Purpose

Add a deterministic client stdio session runtime over an abstract transport.

## Product Context

The codec can render and parse one frame. A client needs session state, request id tracking, frame limits, and deterministic summaries before process IO is introduced.

## Scope

This spec delivers:

- transport trait for sending and receiving line frames
- session state and summary contracts
- request id allocation and matching
- in-memory fixture transport tests for request/response exchange

Owned by other specs:

- spawned process transport
- typed MCP initialize/tools/list methods

## Architecture

`ruse-mcp::client_stdio` owns the session runtime. The transport trait is async so both fixtures and process IO can implement it.

## Public Contracts

Expected contracts:

```rust
#[async_trait]
pub trait McpClientStdioTransport {
    async fn send_frame(&mut self, frame: String) -> Result<(), McpError>;
    async fn recv_frame(&mut self) -> Result<Option<String>, McpError>;
}

pub struct McpClientStdioSession<T> {
    pub transport: T,
}
```

## State and Failure Handling

- Session statuses: created, initialized, failed, closed.
- A request with no response before EOF is a transport error.
- A response with a mismatched id is rejected.
- Close is idempotent and records a bounded reason.

## Security and Privacy

- Summary and error diagnostics do not include raw frames.
- Malformed input from a transport remains untrusted.

## Testing Strategy

- send request and receive matching success
- reject mismatched response id
- reject EOF while waiting for a response
- summary counts sent, received, errors, and close state

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_session`

## Completion Signal

M43-02 is complete when an in-memory transport can drive deterministic request/response session behavior.
