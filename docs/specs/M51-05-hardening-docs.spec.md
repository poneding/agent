# M51-05 Hardening Docs Spec

## Purpose

Document the completed M51 OAuth authorization foundation and record final verification evidence.

## Product Context

M51 adds protocol-adjacent authorization foundations, not a full login product. README and milestone status must describe exactly what works and what remains later work.

## Scope

This spec delivers:

- README update for MCP HTTP OAuth authorization behavior
- milestone status update for M51 completion
- final focused test evidence
- final workspace quality gate evidence

Owned by later specs:

- browser login and Desktop auth UX
- token storage, refresh, revocation, and DCR execution
- provider-specific OAuth flows

## Architecture

Documentation-only hardening over runtime changes in `ruse-mcp`.

## Public Contracts

No new runtime contracts are introduced by this slice.

## State and Failure Handling

- Docs must not claim token acquisition, token refresh, DCR execution, or Desktop auth UX.
- Verification evidence must use `target/codex-m51`.

## Security and Privacy

- Documentation must preserve redaction language for bearer tokens, authorization headers, client secrets, refresh tokens, raw metadata bodies, and remote auth failures.

## Testing Strategy

- final M51 focused tests
- `git diff --check`
- `cargo fmt --all -- --check`
- `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_metadata`
- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_discovery`
- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_authorization_diagnostics`
- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_bearer_authorization`
- `CARGO_TARGET_DIR=target/codex-m51 cargo xtask ci`

## Completion Signal

M51-05 is complete when docs, milestone status, plan evidence, and final gates prove M51 is complete.
