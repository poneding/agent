# M41-03 Sampling Bridge Redaction Spec

## Purpose

Harden MCP sampling model bridge errors and debug surfaces so prompt text, response text, media payloads, provider metadata, and secret-like values do not leak.

## Product Context

M41-02 introduces real model provider execution through the bridge. Redaction must be explicit before wider transport or UI integration.

## Scope

This spec delivers:

- bounded public bridge diagnostics
- secret-like token redaction
- stable diagnostic codes for mapping and provider failures
- tests proving prompts and model output are not present in errors

Owned by later specs:

- audit event summaries
- Desktop trace presentation
- approval redaction UI

## Architecture

Redaction helpers live in `ruse-core::mcp_sampling` because they protect bridge-owned diagnostics. They do not alter MCP wire payloads or model provider inputs.

## Public Contracts

```rust
pub struct McpSamplingBridgeDiagnostic {
    pub code: String,
    pub message: String,
}
```

## State and Failure Handling

- Diagnostic messages are bounded to 256 chars.
- Secret-like values are replaced with `[redacted]`.
- Provider errors are summarized by kind/code.

## Security and Privacy

- No raw prompt, response, system prompt, media, or secret-like string appears in public errors.

## Testing Strategy

- unsupported content error redaction
- provider error redaction
- debug formatting redaction
- stable diagnostic code assertions

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m41 cargo test -p ruse-core mcp_sampling_model_redaction`

## Completion Signal

M41-03 is complete when bridge diagnostics are stable, bounded, and redacted under focused tests.
