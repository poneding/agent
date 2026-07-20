# M52-04 Authorized Request Factory Spec

## Purpose

Compose token leases with M51 bearer request construction so callers can create MCP HTTP POST and GET stream requests only when token status allows it.

## Product Context

M51 applies a raw bearer token. M52 token leases add expiry and scope metadata. This slice ties them together with request factory helpers that enforce expiration and preserve redaction.

## Scope

This spec delivers:

- POST request factory from token lease
- GET stream request factory from token lease
- expired-token rejection before header insertion
- conflict diagnostics for configured authorization headers

Owned by other specs:

- fetching or refreshing token leases
- durable token storage
- Desktop authorization prompts

## Architecture

Factory helpers live in `ruse-mcp::client_http` and call existing `McpClientHttpRequest` / `McpClientHttpStreamRequest` M51 authorization constructors.

## Public Contracts

```rust
pub fn build_mcp_client_http_request_with_oauth_token(...);
pub fn build_mcp_client_http_stream_request_with_oauth_token(...);
```

## State and Failure Handling

- Expired tokens are rejected.
- Near-expiry tokens are allowed but diagnostics remain inspectable through refresh planning.
- Authorization header conflicts use existing redacted errors.

## Security and Privacy

- Token material is only converted into an authorization header at the final request boundary.
- Errors do not expose access-token text.

## Testing Strategy

- POST request with token lease
- GET stream request with token lease and session metadata
- expired token rejection
- header conflict redaction

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p ruse-mcp mcp_client_http_oauth_token_requests`
- fmt and clippy focused gates pass

## Completion Signal

M52-04 is complete when token leases can safely construct authorized MCP HTTP requests through the M51 boundary.
