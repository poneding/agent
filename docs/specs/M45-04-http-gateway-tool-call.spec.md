# M45-04 HTTP Gateway Tool Call Spec

## Purpose

Add live HTTP MCP `tools/call` execution through the existing `McpGatewayClient` and registered local tool path.

## Product Context

After HTTP discovery works, the runtime must prove HTTP MCP tools are executable through the same local policy and output-bounding boundary as stdio tools.

## Scope

This spec delivers:

- `McpGatewayClient::call_tool` implementation for `McpHttpClient<T>`
- request params `{"name": remote_tool_name, "arguments": input_json}`
- result mapping into `McpToolCallResult`
- gateway discover/register/execute coverage with HTTP fixture transport
- redaction and output limit regression tests

Owned by other specs:

- streaming progress or cancellation protocols
- reentrant callbacks during long-running remote tools
- Desktop MCP management UX

## Architecture

No gateway architecture changes should be needed. HTTP clients implement the same trait as stdio clients, and `McpGateway` continues to perform local argument redaction and output bounding.

## Public Contracts

Expected behavior:

```rust
#[async_trait]
impl<T> McpGatewayClient for McpHttpClient<T>
where
    T: McpClientHttpTransport,
{
    async fn call_tool(&self, request: McpToolCallRequest) -> Result<McpToolCallResult, McpError>;
}
```

## State and Failure Handling

- Calls require initialized server id.
- Wrong server id fails before transport.
- JSON-RPC failures expose safe codes only.
- Malformed success results return safe invalid-result diagnostics.
- Gateway output bounding remains local.

## Security and Privacy

- Local policy gates the registered gateway tool before remote HTTP `tools/call`.
- Local tool arguments are redacted before dispatch through gateway.
- Public errors do not expose raw remote failure messages or request bodies.

## Testing Strategy

- HTTP fixture transport tool-call tests
- gateway discover/register/execute test
- malformed result redaction test
- local argument redaction inspection test
- output bounding test

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_gateway_http_tool_call`
- `CARGO_TARGET_DIR=target/codex-m45 cargo clippy -p agent-mcp --all-targets -- -D warnings`

## Completion Signal

M45-04 is complete when an HTTP-backed MCP tool can be registered into the local tool registry and executed safely through `McpGateway`.
