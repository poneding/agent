# M53-02 Authorization Request Planning Spec

## Purpose

Build deterministic authorization request plans for MCP HTTP OAuth flows.

## Product Context

CLI, TUI, Desktop, and scheduler surfaces need a safe authorization URL and flow summary they can present or execute externally. This slice composes M51 metadata with M53 PKCE/state contracts.

## Scope

This spec delivers:

- authorization request input DTO
- authorization URL plan with stable query ordering
- required parameter validation for endpoint, client id, redirect URI, scopes, state, and PKCE challenge
- safe display diagnostics

Owned by other specs:

- browser launch
- localhost callback server
- callback parsing and token exchange
- dynamic client registration

## Architecture

Pure helpers in `agent-mcp::client_http` consume OAuth metadata, client id, redirect URI, requested scopes, PKCE challenge, and state. The result is a UI-safe plan.

## Public Contracts

```rust
pub struct McpClientHttpOAuthAuthorizationRequestInput {
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

pub struct McpClientHttpOAuthAuthorizationRequestPlan {
    pub authorization_url: String,
    pub state_id: String,
    pub code_challenge_method: String,
    pub scopes: Vec<String>,
}
```

## State and Failure Handling

- Missing authorization endpoint is a configuration error.
- Invalid redirect/client/scope values fail before URL construction.
- Query parameters are rendered deterministically for tests and replay.
- No runtime state is mutated.

## Security and Privacy

- Plans do not contain verifier values.
- Errors redact secret-like client ids, redirect values, scopes, and endpoint values.
- The authorization URL is intended for navigation and may be shown to users.

## Testing Strategy

- stable URL construction test
- missing endpoint test
- invalid redirect/client/scope redaction tests
- no verifier leakage tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_authorization_request`
- fmt and clippy focused gates pass

## Completion Signal

M53-02 is complete when authorization request plans are deterministic, typed, and verifier-safe.
