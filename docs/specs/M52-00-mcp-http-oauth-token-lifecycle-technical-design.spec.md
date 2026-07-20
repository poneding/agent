# M52-00 MCP HTTP OAuth Token Lifecycle Technical Design Spec

## Purpose

Define the implementation boundary for MCP HTTP OAuth token lifecycle foundations.

## Product Context

M51 introduced metadata, authorization readiness, and explicit bearer application. M52 adds typed token leases and refresh-aware diagnostics so callers can make safe authorization decisions before Desktop login UX or durable token vaults exist.

## Scope

This spec delivers:

- token lease ownership model
- token source reference model
- refresh planning model
- request factory composition over the M51 bearer boundary
- redaction and testing requirements

Owned by other specs:

- browser/device login
- provider refresh network requests
- durable encrypted token storage
- account selection and Desktop auth prompts

## Architecture

`ruse-mcp::client_http` remains the owner of MCP HTTP auth runtime contracts. It accepts either caller-provided token leases or token source references. It never stores raw token material durably.

## Public Contracts

- `McpClientHttpOAuthTokenLease`
- `McpClientHttpOAuthTokenStatus`
- `McpClientHttpOAuthTokenSourceRef`
- `McpClientHttpOAuthRefreshPlan`
- authorized POST/GET stream request factory helpers

## State and Failure Handling

- Token lease validation rejects empty/control-character token material.
- Expired tokens produce errors and are not applied.
- Near-expiry tokens produce warnings and refresh planning diagnostics.
- Missing scopes produce warnings unless a caller-provided required scope is absent.

## Security and Privacy

- Public diagnostics do not include raw access tokens, refresh tokens, authorization header values, client secrets, raw auth bodies, or remote auth failures.
- Token source names are validated as identifiers and are safe to display.
- `Debug` for token-bearing types is redacted.

## Testing Strategy

- focused tests per slice
- serde stability for non-secret DTOs
- redaction regression tests
- final M52 `cargo xtask ci`

## Acceptance Criteria

- M52 slice specs and plans are complete.
- M52 implementation can proceed one slice at a time without re-scoping the milestone.

## Completion Signal

Technical design is complete when ownership, public contracts, out-of-scope auth work, and verification gates are explicit.
