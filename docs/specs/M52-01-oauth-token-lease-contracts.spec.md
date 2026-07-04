# M52-01 OAuth Token Lease Contracts Spec

## Purpose

Add typed OAuth access-token lease contracts for MCP HTTP authorization without durable token storage.

## Product Context

The M51 bearer boundary accepts a raw caller-provided token. The runtime now needs a safer token-bearing value that carries expiry and scope metadata, has redacted debug behavior, and can be inspected before request construction.

## Scope

This spec delivers:

- `McpClientHttpOAuthTokenLease`
- token lease validation
- token status DTOs and diagnostics
- stable serde for non-secret status reports
- redaction tests for token-bearing contracts

Owned by other specs:

- token source references in M52-02
- refresh planning in M52-03
- request factories in M52-04

## Architecture

Contracts live in `agent-mcp::client_http` next to existing M51 authorization contracts.

## Public Contracts

```rust
pub struct McpClientHttpOAuthTokenLease {
    pub access_token: String,
    pub expires_at_unix_ms: Option<u64>,
    pub scopes: Vec<String>,
}
```

## State and Failure Handling

- Empty and control-character tokens are rejected.
- Empty or whitespace scopes are rejected.
- Missing expiry is accepted but reported as unknown lifetime.

## Security and Privacy

- `Debug` for token leases redacts token material.
- Public status reports do not serialize access-token values.

## Testing Strategy

- valid token lease tests
- invalid token and scope tests
- stable status serde tests
- debug redaction regression tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token`
- `CARGO_TARGET_DIR=target/codex-m52 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m52 cargo clippy -p agent-mcp --all-targets -- -D warnings`

## Completion Signal

M52-01 is complete when token leases and token status reports are typed, redacted, and focused-test verified.
