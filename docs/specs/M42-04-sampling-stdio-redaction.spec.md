# M42-04 Sampling Stdio Redaction Spec

## Purpose

Harden MCP sampling stdio public errors so prompt text, provider failure details, media payloads, and secret-like values do not leak.

## Product Context

M42 accepts untrusted JSON-RPC frames and calls local sampling providers. Public error frames must be safe before this path can be embedded in a live transport runtime.

## Scope

This spec delivers:

- bounded redacted JSON-RPC error messages for sampling stdio dispatch
- provider error redaction tests
- invalid frame redaction tests
- stable diagnostic code assertions

Owned by later specs:

- audit event summaries
- Desktop review surfaces
- live process logs

## Architecture

Redaction helpers live in the M42 `agent-mcp` sampling stdio module and may reuse existing MCP server redaction conventions.

## Public Contracts

No new durable contract is required unless diagnostics become a public struct. Error frames must remain JSON-RPC-compatible.

## State and Failure Handling

- Error message length is bounded.
- Secret-like values become `[redacted]`.
- Provider failures do not expose provider response text.

## Security and Privacy

- No prompt, system prompt, media payload, provider secret, tool input, or local secret name appears in public error frames.

## Testing Strategy

- provider failure redaction
- invalid sampling request redaction
- malformed frame redaction
- bounded error message regression

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_redaction`

## Completion Signal

M42-04 is complete when public sampling stdio error frames are stable, bounded, and redacted.
