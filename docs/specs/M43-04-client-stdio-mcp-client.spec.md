# M43-04 Client Stdio McpClient Spec

## Purpose

Implement the existing `McpClient` trait over the M43 stdio session for initialize and tools/list.

## Product Context

The runtime already has a fixture `McpClient`. M43-04 makes live stdio servers usable through the same typed client boundary without requiring CLI or gateway code to know frame details.

## Scope

This spec delivers:

- stdio-backed `McpClient` implementation
- typed initialize result mapping
- typed tools/list result mapping
- health-check compatibility through existing helper functions

Owned by other specs:

- tools/call gateway execution
- roots, sampling, and reentrant callbacks over live transport

## Architecture

`agent-mcp` maps JSON-RPC result values into existing `McpInitializeResult` and `McpToolListResult` contracts. `McpClient` consumers stay provider-agnostic.

## Public Contracts

Expected contracts:

```rust
pub struct McpStdioClient<T> {
    pub session: McpClientStdioSession<T>,
}

#[async_trait]
impl<T> McpClient for McpStdioClient<T>
where
    T: McpClientStdioTransport + Send + Sync,
{
    async fn initialize(&self, config: &McpServerConfig) -> Result<McpInitializeResult, McpError>;
    async fn list_tools(&self, server_id: &McpServerId) -> Result<McpToolListResult, McpError>;
}
```

Final mutability or locking shape may differ if required by the transport trait.

## State and Failure Handling

- `list_tools` requires a successful initialize in the session.
- Result mapping rejects missing fields, wrong types, and secret-like diagnostic details safely.
- Existing `check_mcp_server_health` works with the stdio client.

## Security and Privacy

- Tool descriptors are metadata, not permission grants.
- Remote schemas are untrusted JSON and remain data until local policy maps a tool.
- Public mapping errors are bounded and redacted.

## Testing Strategy

- initialize maps server result
- tools/list maps descriptors
- health reports healthy for fixture process
- invalid response shape returns redacted error

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_mcp_client`

## Completion Signal

M43-04 is complete when live stdio initialize and tools/list are available through the existing `McpClient` trait.
