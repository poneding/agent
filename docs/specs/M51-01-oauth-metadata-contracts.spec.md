# M51-01 OAuth Metadata Contracts Spec

## Purpose

Add pure OAuth authorization metadata contracts for MCP HTTP clients.

## Product Context

Before a protected MCP HTTP server can be diagnosed or authorized, the runtime needs typed metadata describing issuer, endpoints, registration support, scopes, and supported auth methods.

## Scope

This spec delivers:

- OAuth authorization server metadata DTO
- protected resource metadata DTO
- URL and scope validation helpers
- stable serde shape tests
- redacted invalid-config errors

Owned by other specs:

- metadata discovery transport in M51-02
- policy diagnostics in M51-03
- bearer-token request construction in M51-04

## Architecture

Contracts live in `ruse-mcp::client_http` and are exported through `ruse-mcp`.

## Public Contracts

```rust
pub struct McpClientHttpOAuthMetadata {
    pub issuer: Option<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    pub registration_endpoint: Option<String>,
    pub scopes_supported: Vec<String>,
}
```

## State and Failure Handling

- URL fields must be absent or `http://` / `https://`.
- Scopes must be non-empty trimmed tokens.
- Errors are public and redacted.

## Security and Privacy

- Metadata contracts contain no tokens or client secrets.
- Validation errors do not echo raw metadata values.

## Testing Strategy

- valid metadata contract tests
- invalid URL tests
- invalid scope tests
- stable serde tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_metadata`
- `CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p ruse-mcp --all-targets -- -D warnings`

## Completion Signal

M51-01 is complete when OAuth metadata contracts are typed, exported, validated, redacted, and covered by focused tests.
