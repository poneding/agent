# M53-01 PKCE State Contracts Spec

## Purpose

Add PKCE verifier/challenge and OAuth state contracts for MCP HTTP authorization flows.

## Product Context

Authorization request planning and callback validation both require trustworthy flow-bound values. This slice creates the pure contracts and validation rules before URL planning or token exchange DTOs exist.

## Scope

This spec delivers:

- PKCE verifier contract
- S256 PKCE challenge contract
- OAuth state contract
- redacted `Debug` for verifier/state-bearing types
- stable serialization for non-secret challenge metadata

Owned by other specs:

- authorization URL planning
- callback validation
- token exchange requests
- random generation and durable flow storage

## Architecture

Pure contracts live in `ruse-mcp::client_http`. Callers provide verifier and state values from a secure random source outside `ruse-mcp`.

## Public Contracts

```rust
pub struct McpClientHttpOAuthPkceVerifier {
    pub verifier: String,
}

pub struct McpClientHttpOAuthPkceChallenge {
    pub challenge: String,
    pub method: String,
}

pub struct McpClientHttpOAuthState {
    pub state: String,
}
```

## State and Failure Handling

- PKCE verifier values must follow RFC 7636 length and character rules.
- State values must be non-empty, bounded, printable, and secret-safe.
- Invalid values return redacted `InvalidConfig` errors.
- Challenge generation is deterministic for a verifier.

## Security and Privacy

- Verifier and state values are treated as sensitive flow material.
- `Debug` redacts verifier and state values.
- Errors never echo invalid verifier or state input.
- `McpClientHttpOAuthPkceVerifier` and `McpClientHttpOAuthState` are not durable DTOs.

## Testing Strategy

- valid verifier/state construction tests
- invalid verifier/state rejection tests
- S256 challenge deterministic known-vector test
- redacted debug and error tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_pkce_state`
- fmt and clippy focused gates pass

## Completion Signal

M53-01 is complete when PKCE/state contracts are typed, deterministic, redacted, and focused-test verified.
