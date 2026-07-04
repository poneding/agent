# M66-04 OAuth Completion for Installable MCP Workflows Spec

## Purpose

Confirm whether M66 needs additional MCP OAuth implementation for installable MCP workflows. The answer for this slice is: not yet. Existing OAuth runtime foundations are sufficient for the M66 plugin/skill/MCP health workflows completed so far, and no current installable workflow requires new browser-loop, callback-server, token-storage, or refresh-execution code.

This is an explicit product decision, not a forgotten gap. M66 remains focused on extension management hardening rather than protocol expansion.

## Evidence

Existing `agent-mcp` foundations already cover:

- OAuth metadata parsing and validation
- protected resource metadata
- PKCE verifier/challenge and state contracts
- authorization request planning
- callback validation and state mismatch handling
- authorization-code token exchange request construction
- token exchange transport and runner diagnostics
- token response parsing
- token lease conversion
- token source and refresh planning diagnostics
- bearer authorization construction
- redacted debug and JSON diagnostics

Relevant tests:

- `crates/agent-mcp/tests/mcp_client_http_oauth_metadata.rs`
- `crates/agent-mcp/tests/mcp_client_http_oauth_pkce_state.rs`
- `crates/agent-mcp/tests/mcp_client_http_oauth_authorization_request.rs`
- `crates/agent-mcp/tests/mcp_client_http_oauth_callback.rs`
- `crates/agent-mcp/tests/mcp_client_http_oauth_token_exchange_runner.rs`
- `crates/agent-mcp/tests/mcp_client_http_oauth_token_exchange_lease.rs`
- `crates/agent-mcp/tests/mcp_client_http_oauth_token_refresh.rs`
- `crates/agent-mcp/tests/mcp_client_http_bearer_authorization.rs`

## Current M66 Workflow Need

M66-01 through M66-03 need:

- extension health baseline
- plugin and skill management hardening
- MCP status and compatibility health
- redacted diagnostics
- stable CLI/JSON surfaces

They do not require:

- launching a browser
- running a local callback HTTP server
- persisting OAuth refresh tokens
- executing refresh requests in background
- adding Desktop OAuth setup UI
- collecting token material in support bundles

## Deferral

The following remain deferred until an installable MCP workflow explicitly needs them:

- Desktop/CLI OAuth setup wizard
- localhost callback listener
- secure token persistence and migration
- refresh-token execution and retry scheduling
- support bundle OAuth readiness evidence beyond redacted status
- live OAuth smoke requiring a real provider

## Security Requirements

When OAuth product work resumes:

- token values must never appear in CLI, Desktop, bridge frames, logs, support bundles, or tests
- callback codes and PKCE verifiers must remain redacted in debug output
- refresh tokens must only be stored through the repository's secret boundary
- support evidence may include status, diagnostic code, expiry class, and required scopes, never raw token material

## Acceptance Criteria

- M66 documents that no additional OAuth code is required for current installable workflows.
- Existing OAuth tests are identified as coverage for the current foundation.
- M66 master plan marks OAuth completion as intentionally deferred unless a later M66 slice discovers a blocking workflow.
- M66-05 support evidence must avoid raw OAuth material and may only include redacted status if added.

## Verification

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-mcp oauth
cargo fmt --all -- --check
git diff --check
```
