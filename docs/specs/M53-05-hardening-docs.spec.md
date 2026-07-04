# M53-05 Hardening Docs Spec

## Purpose

Document the completed M53 OAuth authorization-flow foundation and record final verification evidence.

## Product Context

M53 should make OAuth flow setup and callback handoff safer without claiming browser login, token endpoint execution, provider-specific refresh, or durable token storage.

## Scope

This spec delivers:

- README update for OAuth authorization-flow behavior
- milestone status update for M53 completion
- final focused test evidence
- final workspace quality gate evidence

Owned by later specs:

- browser/device login
- Desktop account UX
- durable encrypted token vault
- provider-specific token exchange, refresh, revocation, and DCR execution

## Architecture

Documentation-only hardening over runtime changes in `agent-mcp`.

## Public Contracts

No new runtime contracts are introduced by this slice.

## State and Failure Handling

- Docs must not claim token acquisition, network token exchange execution, or durable token storage.
- Verification evidence must use `target/codex-m53`.

## Security and Privacy

- Documentation must preserve redaction language for PKCE verifiers, state values, authorization codes, callback queries, access tokens, refresh tokens, authorization headers, client secrets, raw auth bodies, and remote auth failures.

## Testing Strategy

- final M53 focused tests
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_pkce_state`
- `CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_authorization_request`
- `CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_callback`
- `CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_flow_diagnostics`
- `CARGO_TARGET_DIR=target/codex-m53 cargo xtask ci`

## Completion Signal

M53-05 is complete when docs, milestone status, plan evidence, and final gates prove M53 is complete.
