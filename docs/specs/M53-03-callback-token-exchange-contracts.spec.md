# M53-03 Callback Token Exchange Contracts Spec

## Purpose

Validate OAuth callback outcomes and build token exchange request DTOs without executing network calls.

## Product Context

After a UI or external browser receives an OAuth callback, the runtime must prove state integrity before any token exchange can be attempted. This slice creates that auditable handoff.

## Scope

This spec delivers:

- callback input DTO
- success/error/state-mismatch validation result
- token exchange request DTO for authorization code grant with PKCE
- redacted debug and error behavior

Owned by other specs:

- callback listener implementation
- HTTP token exchange execution
- token response parsing
- durable token storage

## Architecture

Pure helpers in `ruse-mcp::client_http` validate callback values against expected state and compose a token endpoint form request DTO from validated code plus PKCE verifier.

## Public Contracts

```rust
pub struct McpClientHttpOAuthCallbackInput {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

pub struct McpClientHttpOAuthTokenExchangeRequest {
    pub token_endpoint: String,
    pub form: Vec<(String, String)>,
}
```

## State and Failure Handling

- Provider error callbacks are accepted as terminal diagnostics without token exchange.
- State mismatch rejects the callback and redacts all callback fields.
- Missing authorization code rejects token exchange.
- Token exchange DTO construction requires validated success callback, token endpoint, client id, redirect URI, and PKCE verifier.

## Security and Privacy

- Authorization codes, state values, verifier values, and provider error descriptions do not appear in public diagnostics.
- Token exchange request DTO is a final boundary object and must not be persisted as trace or config.
- Debug for token exchange DTO redacts form values.

## Testing Strategy

- callback success test
- state mismatch test
- provider error test
- token exchange form shape test
- redaction tests for code/state/verifier/error description

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_callback`
- fmt and clippy focused gates pass

## Completion Signal

M53-03 is complete when callbacks are state-validated and successful flows can produce redacted token exchange DTOs without network execution.
