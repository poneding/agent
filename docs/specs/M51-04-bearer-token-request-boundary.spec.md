# M51-04 Bearer Token Request Boundary Spec

## Purpose

Add explicit bearer-token application to MCP HTTP request builders for POST JSON-RPC calls and GET stream requests.

## Product Context

M51 intentionally does not acquire tokens. It still needs a narrow, testable boundary where caller-provided authorization material is applied to HTTP requests without leaking through diagnostics.

## Scope

This spec delivers:

- bearer authorization DTO
- POST request builder variant with authorization
- GET stream request builder variant with authorization
- conflict handling when authorization headers are already configured
- redaction tests

Owned by other specs:

- token storage and refresh
- browser or Desktop auth UX

## Architecture

Authorization application lives next to existing `McpClientHttpRequest` and `McpClientHttpStreamRequest` constructors.

## Public Contracts

```rust
pub struct McpClientHttpAuthorization {
    pub bearer_token: String,
}
```

## State and Failure Handling

- Empty bearer tokens are rejected.
- Tokens are never stored on `McpHttpClient`.
- Existing `Authorization` headers conflict with explicit authorization.
- Errors are redacted.

## Security and Privacy

- Public reports never serialize token material.
- Header values are not echoed in diagnostics.

## Testing Strategy

- POST request authorization header test
- GET stream authorization header test
- empty token rejection
- header conflict rejection
- redaction regression

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_bearer_authorization`
- `CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p ruse-mcp --all-targets -- -D warnings`

## Completion Signal

M51-04 is complete when caller-provided bearer authorization can be applied to POST and GET stream requests safely and explicitly.
