# M44-01 Stdio Tool Call Mapping Spec

## Purpose

Implement `McpStdioClient` support for serializing MCP `tools/call` requests and mapping success responses into `McpToolCallResult`.

## Product Context

Live stdio clients can already initialize and list tools. This slice adds the core typed operation required before gateway E2E execution can use remote tools.

## Scope

This spec delivers:

- `McpGatewayClient` implementation for `McpStdioClient<T>`
- request params mapping for `tools/call`
- success result mapping into `McpToolCallResult`
- focused fixture tests for request shape and mapping

Owned by other specs:

- gateway live stdio E2E
- subprocess helper coverage
- hardening docs

## Architecture

`ruse-mcp::client_stdio` owns the implementation. It reuses `McpClientStdioSession::send_request` and `McpServerMethod::ToolsCall`.

## Public Contracts

```rust
#[async_trait]
impl<T> McpGatewayClient for McpStdioClient<T>
where
    T: McpClientStdioTransport,
{
    async fn call_tool(&self, request: McpToolCallRequest) -> Result<McpToolCallResult, McpError>;
}
```

## State and Failure Handling

- Calling before initialize returns a client error.
- Calling a different server id returns a client error before transport writes.
- JSON-RPC failure frames are converted to bounded client errors.
- Success result is preserved as `McpToolCallResult::new(result)`.

## Security and Privacy

- The client does not log or expose raw arguments in errors.
- Remote error messages are not surfaced in public diagnostics.

## Testing Strategy

- fixture transport verifies frame method, params name, and arguments
- fixture transport verifies success result mapping
- fixture transport verifies uninitialized call rejection

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m44 cargo test -p ruse-mcp mcp_client_stdio_tool_call`

## Completion Signal

M44-01 is complete when a fixture-backed stdio client can call a remote tool through `McpGatewayClient`.
