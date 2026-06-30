# M51-03 Authorization Policy Diagnostics Spec

## Purpose

Add redacted diagnostics that explain whether an MCP HTTP server appears to require authorization and whether configured metadata is usable.

## Product Context

Metadata alone is not enough for CLI/Desktop users. They need a safe summary of authorization readiness without seeing tokens, headers, or raw remote bodies.

## Scope

This spec delivers:

- authorization diagnostic severity enum
- authorization readiness report DTO
- redacted summaries for missing token endpoint, missing scopes, and header conflicts

Owned by other specs:

- actual token acquisition
- Desktop authorization prompts

## Architecture

Diagnostics live in `agent-mcp::client_http` and are pure functions over config plus metadata DTOs.

## Public Contracts

```rust
pub struct McpClientHttpAuthorizationReport {
    pub authorized: bool,
    pub diagnostics: Vec<McpClientHttpAuthorizationDiagnostic>,
}
```

## State and Failure Handling

- Diagnostics do not mutate client state.
- Missing metadata produces warnings.
- conflicting local authorization headers produce errors.

## Security and Privacy

- Reports expose no bearer token, authorization header value, client secret, raw metadata body, or remote auth error.

## Testing Strategy

- stable report serde tests
- missing metadata diagnostics
- header conflict diagnostics
- redaction regressions

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p agent-mcp mcp_client_http_authorization_diagnostics`
- `CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p agent-mcp --all-targets -- -D warnings`

## Completion Signal

M51-03 is complete when authorization readiness diagnostics are typed, stable, redacted, and fixture-tested.
