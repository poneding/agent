# M53-00 MCP HTTP OAuth Authorization Flow Technical Design Spec

## Purpose

Define the implementation boundary for MCP HTTP OAuth authorization-flow foundations.

## Product Context

M51 introduced OAuth metadata and bearer application. M52 introduced token leases and refresh-aware request factories. M53 creates the missing pre-token flow layer so user-facing clients can safely initiate and validate OAuth authorization code flows later.

## Scope

This spec delivers:

- PKCE verifier and challenge ownership model
- state nonce and flow reference model
- authorization request planning model
- callback validation model
- token exchange request DTO boundary
- redaction and testing requirements

Owned by other specs:

- browser/device UX
- localhost callback server lifecycle
- token endpoint network execution
- durable encrypted token storage
- dynamic client registration, refresh, and revocation execution

## Architecture

`ruse-mcp::client_http` remains the owner of MCP HTTP auth runtime contracts. Flow helpers are pure and accept caller-provided entropy, metadata, redirect URI, client id, requested scopes, and callback fields. UI and storage layers consume flow plans and own side effects.

## Public Contracts

- `McpClientHttpOAuthPkceVerifier`
- `McpClientHttpOAuthPkceChallenge`
- `McpClientHttpOAuthState`
- `McpClientHttpOAuthAuthorizationRequestPlan`
- `McpClientHttpOAuthCallbackValidation`
- `McpClientHttpOAuthTokenExchangeRequest`
- `McpClientHttpOAuthFlowDiagnostic`

## State and Failure Handling

- Invalid PKCE verifiers, state values, redirect URIs, client ids, or scopes are rejected with redacted config errors.
- Authorization request planning requires an OAuth authorization endpoint.
- Callback validation requires exact state match before exposing an authorization code to later exchange steps.
- Token exchange DTO construction requires a validated success callback and token endpoint.
- All functions are deterministic and idempotent for identical inputs.

## Security and Privacy

- Verifiers, authorization codes, state values, callback queries, provider error descriptions, client secrets, token responses, and token endpoint request bodies never appear in public errors.
- `Debug` for secret-bearing flow types is redacted.
- Public DTOs intended for UI navigation contain only user-safe fields.
- `ruse-mcp` does not store flow state durably.

## Testing Strategy

- focused tests per slice
- stable serialization tests for non-secret DTOs
- redaction regression tests
- final M53 `cargo xtask ci`

## Acceptance Criteria

- M53 slice specs and plans are complete.
- M53 implementation can proceed one slice at a time without re-scoping the milestone.

## Completion Signal

Technical design is complete when ownership, public contracts, out-of-scope auth work, redaction rules, and verification gates are explicit.
