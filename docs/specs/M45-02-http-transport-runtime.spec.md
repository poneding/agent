# M45-02 HTTP Transport Runtime Spec

## Purpose

Add the transport abstraction and initial runtime implementation needed to send MCP JSON-RPC request bodies to streamable HTTP server URLs.

## Product Context

M45-01 provides pure contracts. This slice introduces the I/O boundary while keeping network behavior mockable and secret-safe.

## Scope

This spec delivers:

- `McpClientHttpTransport` trait
- deterministic fixture transport
- live transport constructor for `McpTransportConfig::StreamableHttp`
- header validation and resolved-secret handling
- timeout and HTTP status diagnostics

Owned by other specs:

- typed initialize/tools/list mapping
- gateway tools/call execution
- SSE/resumable stream support

## Architecture

Transport behavior remains behind a trait so tests can use in-memory responses. A live transport may use an existing workspace HTTP client dependency if consistent with local patterns.

## Public Contracts

Expected shape:

```rust
#[async_trait]
pub trait McpClientHttpTransport: Send + Sync {
    async fn send_request(
        &self,
        request: McpClientHttpRequest,
    ) -> Result<McpClientHttpResponse, McpError>;
}
```

## State and Failure Handling

- Disabled server configs are rejected before transport construction.
- Non-HTTP configs are rejected before transport construction.
- Unresolved secret refs in headers fail before transport construction.
- Non-2xx HTTP status returns a bounded safe error.
- Timeouts return a bounded safe error.

## Security and Privacy

- Header values and secret ref names do not appear in public diagnostics.
- URLs are redacted before display.
- Response bodies from failed HTTP statuses are not surfaced directly.

## Testing Strategy

- fixture transport tests
- config rejection tests
- header secret-ref rejection tests
- timeout/status redaction tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_transport`
- `CARGO_TARGET_DIR=target/codex-m45 cargo clippy -p agent-mcp --all-targets -- -D warnings`

## Completion Signal

M45-02 is complete when HTTP transport behavior is constructible, mockable, and safely fails before higher-level MCP mapping.
