# M45-03 HTTP McpClient Discovery Spec

## Purpose

Implement `McpClient` over the streamable HTTP transport for initialize and tools/list.

## Product Context

The gateway depends on `McpClient` health checks and tool discovery before it can register remote tools. HTTP servers need the same typed discovery path as stdio servers.

## Scope

This spec delivers:

- `McpHttpClient<T>` session state
- `initialize` over HTTP JSON-RPC
- `tools/list` over HTTP JSON-RPC
- `McpClientRootProvider` empty root behavior until HTTP roots are explicitly expanded
- health check compatibility through existing `check_mcp_server_health`

Owned by other specs:

- `tools/call` over HTTP
- advanced stream/session negotiation
- roots/list HTTP server-requested callbacks

## Architecture

`McpHttpClient` should reuse M43/M44 mapping rules where possible and keep result shape compatibility with existing gateway code.

## Public Contracts

Expected behavior:

```rust
#[async_trait]
impl<T> McpClient for McpHttpClient<T>
where
    T: McpClientHttpTransport,
{
    async fn initialize(&self, config: &McpServerConfig) -> Result<McpInitializeResult, McpError>;
    async fn list_tools(&self, server_id: &McpServerId) -> Result<McpToolListResult, McpError>;
}
```

## State and Failure Handling

- Initialize must happen before list.
- Wrong server id fails before transport call.
- Mapping failures use redacted diagnostics.
- JSON-RPC failure frames expose safe operation and code only.

## Security and Privacy

- Initialize and tools/list diagnostics do not expose raw remote response bodies.
- Tool descriptors are metadata only.
- Discovery does not grant filesystem, command, Git, browser, Desktop, secret, or local tool permissions.

## Testing Strategy

- fixture transport initialize tests
- fixture transport tools/list tests
- wrong-server and uninitialized tests
- health check test
- malformed result redaction test

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_mcp_client`
- `CARGO_TARGET_DIR=target/codex-m45 cargo clippy -p agent-mcp --all-targets -- -D warnings`

## Completion Signal

M45-03 is complete when HTTP MCP servers can be initialized and listed through the existing typed client boundary.
