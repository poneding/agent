# M54-02 Token Exchange Transport Spec

## Purpose

Add an async token exchange transport boundary for OAuth token endpoint form POST execution.

## Product Context

The runtime needs a deterministic fixture transport for tests and a reqwest-backed transport for real providers, while keeping token form bodies and provider responses out of diagnostics.

## Scope

This spec delivers:

- exchange HTTP request/response DTOs
- async exchange transport trait
- fixture exchange transport
- reqwest form POST implementation
- redaction tests for request, response, and transport failures

Owned by other specs:

- token response parsing
- runner orchestration
- lease conversion

## Architecture

The transport trait is independent from MCP JSON-RPC request transport because OAuth token endpoints use form POST and OAuth JSON response shapes, not MCP envelopes.

## Public Contracts

```rust
pub struct McpClientHttpOAuthTokenExchangeHttpRequest {
    pub token_endpoint: String,
    pub form: Vec<(String, String)>,
}

pub struct McpClientHttpOAuthTokenExchangeHttpResponse {
    pub status: u16,
    pub body: String,
}
```

## State and Failure Handling

- Invalid endpoint URLs and empty form values fail before send.
- Transport errors become redacted `McpError` values.
- Response bodies stay in secret-bearing response DTOs and are not logged by diagnostics.

## Security and Privacy

- `Debug` redacts form values and response body.
- Reqwest transport uses `application/x-www-form-urlencoded`.
- No authorization headers or client secrets are injected by this slice.

## Testing Strategy

- fixture transport success and error tests
- reqwest request-shape tests where practical without external network
- redacted debug tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m54 cargo test -p agent-mcp mcp_client_http_oauth_token_exchange_transport`
- focused fmt and clippy gates pass for `agent-mcp`

## Completion Signal

M54-02 is complete when token exchange HTTP execution has a typed async transport, fixture coverage, reqwest implementation, and redacted diagnostics.
