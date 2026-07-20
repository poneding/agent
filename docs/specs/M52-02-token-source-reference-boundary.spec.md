# M52-02 Token Source Reference Boundary Spec

## Purpose

Add typed token source references so `ruse-mcp` can identify external token material without storing raw tokens.

## Product Context

Future CLI/Desktop/auth layers need to pass references to token material across runtime boundaries. The MCP crate should validate and report those references as safe identifiers, not resolve or persist secrets.

## Scope

This spec delivers:

- token source reference DTO
- validation for source ids and account/resource labels
- safe display and serde shape
- diagnostics for missing or malformed references

Owned by other specs:

- real secret resolver integration
- encrypted token vaults
- account-selection UX

## Architecture

`ruse-mcp::client_http` owns the reference contract only. Resolver crates remain responsible for mapping references to token leases.

## Public Contracts

```rust
pub struct McpClientHttpOAuthTokenSourceRef {
    pub source_id: String,
    pub account_hint: Option<String>,
    pub resource: Option<String>,
}
```

## State and Failure Handling

- Empty source ids are rejected.
- Account/resource hints are optional and bounded.
- Diagnostics do not reveal secret handles that look token-like.

## Security and Privacy

- References are metadata, never token material.
- Secret-like reference text is redacted in errors.

## Testing Strategy

- valid reference serde tests
- invalid source id tests
- redaction regression tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p ruse-mcp mcp_client_http_oauth_token_source`
- fmt and clippy focused gates pass

## Completion Signal

M52-02 is complete when token source references are safe public contracts with validation and redacted diagnostics.
