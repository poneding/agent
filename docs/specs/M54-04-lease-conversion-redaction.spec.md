# M54-04 Lease Conversion Redaction Spec

## Purpose

Convert successful OAuth token responses into existing `McpClientHttpOAuthTokenLease` contracts and harden redaction across token exchange boundaries.

## Product Context

After a successful exchange, callers need a token lease that can flow into M52 token lifecycle checks and request factories. The conversion must keep refresh token and provider response material out of durable contracts.

## Scope

This spec delivers:

- token response to token lease conversion helper
- expiry calculation from caller-provided `now_unix_ms`
- scope parsing and normalization
- redaction regression tests across exchange response, report, and lease conversion

Owned by other specs:

- durable vault storage
- refresh-token persistence and refresh execution
- account UX

## Architecture

Lease conversion stays in `ruse-mcp::client_http` and reuses `McpClientHttpOAuthTokenLease::new`.

## Public Contracts

```rust
pub fn build_mcp_client_http_oauth_token_lease_from_exchange_response(
    response: &McpClientHttpOAuthTokenResponse,
    now_unix_ms: u64,
) -> Result<McpClientHttpOAuthTokenLease, McpError>;
```

## State and Failure Handling

- Bearer token responses convert to leases.
- Unsupported token type fails safely.
- `expires_in_seconds` uses saturating millisecond conversion.
- Missing expiry produces a lease with `None`.

## Security and Privacy

- Refresh tokens are not included in token leases.
- Diagnostics never mention raw scopes if they are invalid and secret-like.
- Debug output remains redacted for response and lease.

## Testing Strategy

- lease conversion with expiry
- lease conversion without expiry
- unsupported token type rejection
- refresh token omission test
- redaction regression tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_lease`
- focused fmt and clippy gates pass for `ruse-mcp`

## Completion Signal

M54-04 is complete when successful token exchange responses can become redacted, usable token leases without durable refresh-token storage.
