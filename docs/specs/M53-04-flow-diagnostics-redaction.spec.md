# M53-04 Flow Diagnostics Redaction Spec

## Purpose

Harden OAuth flow diagnostics so authorization planning, callback validation, and token exchange DTO boundaries remain useful and secret-safe.

## Product Context

OAuth failures often contain provider-specific raw descriptions, callback query values, codes, state values, and endpoint details. CLI and Desktop need actionable status without leaking secrets into logs, traces, or screenshots.

## Scope

This spec delivers:

- flow diagnostic severity and code contracts
- redacted diagnostic builder helpers
- aggregate flow report DTOs for UI-safe summaries
- regression tests for secret-like input and provider error descriptions

Owned by other specs:

- Desktop auth UX rendering
- audit persistence schema
- provider-specific token exchange execution

## Architecture

Diagnostic contracts live in `agent-mcp::client_http` and are reused by M53 flow helpers. They remain serializable because they carry only redacted, bounded public text.

## Public Contracts

```rust
pub struct McpClientHttpOAuthFlowDiagnostic {
    pub severity: McpClientHttpOAuthFlowDiagnosticSeverity,
    pub code: McpClientHttpOAuthFlowDiagnosticCode,
    pub message: String,
}
```

## State and Failure Handling

- Diagnostics classify missing endpoint, invalid input, state mismatch, provider error, and exchange-ready states.
- Terminal callback failures produce error severity.
- Planning warnings never block caller-owned retry unless an error diagnostic exists.

## Security and Privacy

- Diagnostics never include raw authorization codes, verifiers, state values, callback queries, provider descriptions, client secrets, access tokens, refresh tokens, or token endpoint bodies.
- Messages are bounded and deterministic.

## Testing Strategy

- serde stability tests
- redaction tests for every flow boundary
- severity and code tests for terminal vs warning cases

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_flow_diagnostics`
- fmt and clippy focused gates pass

## Completion Signal

M53-04 is complete when OAuth flow reports are serializable, deterministic, and redaction-tested.
