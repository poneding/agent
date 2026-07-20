# M54-05 Hardening Docs Spec

## Purpose

Document the completed M54 token exchange runtime and record final verification evidence.

## Product Context

M54 should make token exchange execution available without claiming browser login, durable token storage, refresh execution, revocation, DCR, or Desktop account UX.

## Scope

This spec delivers:

- README update for token exchange behavior
- milestone status update for M54 completion
- final focused test evidence
- final workspace quality gate evidence

Owned by later specs:

- browser/device login
- Desktop account UX
- durable encrypted token vault
- refresh, revocation, and DCR execution

## Architecture

Documentation-only hardening over runtime changes in `ruse-mcp`.

## Public Contracts

No new runtime contracts are introduced by this slice.

## State and Failure Handling

- Docs must not claim durable storage, refresh execution, revocation, DCR, or account UX.
- Verification evidence must use `target/codex-m54`.

## Security and Privacy

- Documentation must preserve redaction language for authorization codes, PKCE verifiers, state values, access tokens, refresh tokens, client secrets, form bodies, authorization headers, raw provider responses, and remote auth failures.

## Testing Strategy

- final M54 focused tests
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_response`
- `CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_transport`
- `CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_runner`
- `CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_lease`
- `CARGO_TARGET_DIR=target/codex-m54 cargo xtask ci`

## Completion Signal

M54-05 is complete when docs, milestone status, plan evidence, and final gates prove M54 is complete.
