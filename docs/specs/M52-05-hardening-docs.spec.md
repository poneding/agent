# M52-05 Hardening Docs Spec

## Purpose

Document the completed M52 token lifecycle foundation and record final verification evidence.

## Product Context

M52 should make token status and request application safer without claiming end-user login, provider refresh execution, or durable token storage.

## Scope

This spec delivers:

- README update for token lifecycle behavior
- milestone status update for M52 completion
- final focused test evidence
- final workspace quality gate evidence

Owned by later specs:

- browser/device login
- Desktop account UX
- durable encrypted token vault
- provider-specific refresh and revocation execution

## Architecture

Documentation-only hardening over runtime changes in `agent-mcp`.

## Public Contracts

No new runtime contracts are introduced by this slice.

## State and Failure Handling

- Docs must not claim token acquisition or durable token storage.
- Verification evidence must use `target/codex-m52`.

## Security and Privacy

- Documentation must preserve redaction language for access tokens, refresh tokens, authorization headers, client secrets, raw auth bodies, and remote auth failures.

## Testing Strategy

- final M52 focused tests
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token`
- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_source`
- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_refresh`
- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_requests`
- `CARGO_TARGET_DIR=target/codex-m52 cargo xtask ci`

## Completion Signal

M52-05 is complete when docs, milestone status, plan evidence, and final gates prove M52 is complete.
