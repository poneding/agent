# M54-03 Token Exchange Runner Diagnostics Spec

## Purpose

Compose the M53 token exchange request, M54 transport, and M54 token response parser into one redacted exchange runner report.

## Product Context

Callers need a single runtime function that attempts token exchange, distinguishes success, provider error, invalid response, and retryable transport failure, and gives CLI/Desktop safe diagnostics.

## Scope

This spec delivers:

- token exchange diagnostic severity/code contracts
- serializable exchange report
- runner function over the exchange transport trait
- status classification for success, provider error, invalid response, and transport failure

Owned by other specs:

- response contracts
- transport implementation
- token lease conversion
- docs hardening

## Architecture

The runner is async and pure with respect to storage. It mutates no runtime state and returns a report plus optional secret-bearing response.

## Public Contracts

```rust
pub struct McpClientHttpOAuthTokenExchangeReport {
    pub exchanged: bool,
    pub diagnostics: Vec<McpClientHttpOAuthTokenExchangeDiagnostic>,
}
```

## State and Failure Handling

- 2xx responses parse success token responses.
- 4xx provider OAuth errors produce terminal error diagnostics.
- 5xx and transport errors are retryable diagnostics.
- Invalid JSON and unsupported content produce invalid-response diagnostics.

## Security and Privacy

- Reports are safe to serialize.
- Reports never include raw response bodies, form values, tokens, authorization codes, verifiers, or provider descriptions.

## Testing Strategy

- runner success with fixture transport
- provider error classification
- retryable transport failure classification
- invalid response classification
- report serde stability and redaction

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_runner`
- focused fmt and clippy gates pass for `ruse-mcp`

## Completion Signal

M54-03 is complete when token exchange execution yields deterministic safe reports and secret-bearing responses only at explicit boundaries.
