# M54-01 Token Response Contracts Spec

## Purpose

Add typed contracts for OAuth token endpoint success and error responses without adding network execution.

## Product Context

The token exchange runner needs a validated in-memory representation of successful token responses and provider errors before it can safely execute HTTP calls or create token leases.

## Scope

This spec delivers:

- token response DTO for success responses
- provider token error DTO for OAuth error responses
- parser helpers for bounded JSON bodies
- redacted `Debug` and validation behavior

Owned by other specs:

- transport execution
- exchange runner orchestration
- lease conversion
- docs hardening

## Architecture

Contracts live in `agent-mcp::client_http` next to existing OAuth metadata, flow, and token lifecycle contracts.

## Public Contracts

```rust
pub struct McpClientHttpOAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in_seconds: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Vec<String>,
}

pub struct McpClientHttpOAuthTokenErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}
```

## State and Failure Handling

- Missing or empty `access_token` is invalid.
- `token_type` defaults only when the provider explicitly omits it if local policy accepts bearer defaults; otherwise invalid.
- Unsupported token type is invalid for lease conversion.
- Invalid JSON and malformed fields return redacted flow errors.

## Security and Privacy

- Success and error response DTOs are secret-bearing and do not derive `Serialize`.
- `Debug` redacts access token, refresh token, and provider descriptions.
- Parser errors never include raw JSON bodies.

## Testing Strategy

- parse valid bearer response
- parse error response without leaking description
- reject malformed JSON and missing access token
- redacted debug tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m54 cargo test -p agent-mcp mcp_client_http_oauth_token_response`
- focused fmt and clippy gates pass for `agent-mcp`

## Completion Signal

M54-01 is complete when token endpoint response contracts are typed, validated, redacted, exported, and covered by focused tests.
