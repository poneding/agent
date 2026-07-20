# M45-01 HTTP Client Contracts Spec

## Purpose

Add pure, deterministic JSON-RPC HTTP request and response contracts for MCP client streamable HTTP operations.

## Product Context

Before introducing a live HTTP transport, the runtime needs stable body rendering, response parsing, id validation, status handling, and redacted diagnostics that can be tested without network I/O.

## Scope

This spec delivers:

- typed HTTP client request/response frame structures
- request rendering for `initialize`, `tools/list`, and `tools/call`
- response parsing into success/failure frames
- HTTP status and malformed body diagnostics
- redaction tests for remote messages and secret-like payloads

Owned by other specs:

- reqwest or live network transport
- `McpClient` trait implementation
- gateway registration and execution

## Architecture

Add pure helpers under `ruse-mcp`, likely in `client_http.rs`, mirroring the stdio codec separation while returning HTTP-oriented request/response values.

## Public Contracts

Expected helpers:

```rust
pub fn render_mcp_client_http_request_body(
    id: McpJsonRpcId,
    method: impl Into<String>,
    params: serde_json::Value,
) -> Result<serde_json::Value, McpError>;

pub fn parse_mcp_client_http_response_body(
    body: &serde_json::Value,
) -> Result<McpClientHttpResponseFrame, McpError>;
```

## State and Failure Handling

- Request ids cannot be null.
- Method names cannot be empty.
- Responses must be JSON-RPC 2.0.
- Responses must contain exactly one of `result` or `error`.
- Error details remain structured internally but public diagnostics expose safe codes only.

## Security and Privacy

- Raw request params and response bodies must not appear in parse errors.
- Remote error messages are not surfaced directly.
- Bodies above the configured max size are rejected before parsing when size metadata is available.

## Testing Strategy

- request body rendering tests
- success/failure response parse tests
- malformed body tests
- redaction regression tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p ruse-mcp mcp_client_http_contracts`
- `CARGO_TARGET_DIR=target/codex-m45 cargo clippy -p ruse-mcp --all-targets -- -D warnings`

## Completion Signal

M45-01 is complete when pure HTTP client contract tests pass without live network behavior.
