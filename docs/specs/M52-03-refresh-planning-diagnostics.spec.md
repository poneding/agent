# M52-03 Refresh Planning Diagnostics Spec

## Purpose

Add pure refresh planning diagnostics for MCP HTTP OAuth token leases.

## Product Context

Schedulers and UI surfaces need to know whether a token is expired, near expiry, missing expiry, or scope-incompatible before attempting work. This slice creates that planning layer without executing OAuth refresh network calls.

## Scope

This spec delivers:

- refresh plan DTO
- configurable refresh skew
- expired/refresh-soon/unknown-expiry diagnostics
- required-scope compatibility checks

Owned by other specs:

- network refresh execution
- retrying failed refreshes
- storing refreshed tokens

## Architecture

Pure functions in `agent-mcp::client_http` evaluate token leases against caller-provided time and requirements.

## Public Contracts

```rust
pub struct McpClientHttpOAuthRefreshPlan {
    pub action: McpClientHttpOAuthRefreshAction,
    pub diagnostics: Vec<McpClientHttpOAuthTokenDiagnostic>,
}
```

## State and Failure Handling

- Planning is deterministic for a given `now_unix_ms`.
- Expired tokens are terminal for request application.
- Refresh-soon tokens are warnings.

## Security and Privacy

- Plans never serialize token material.
- Scope diagnostics expose requested scope names only after validation.

## Testing Strategy

- expired token plan
- refresh-soon plan
- no-expiry plan
- required-scope missing diagnostics

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_refresh`
- fmt and clippy focused gates pass

## Completion Signal

M52-03 is complete when token refresh planning is typed, deterministic, and redacted.
