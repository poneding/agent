# MX01-03 Sampling Productization Spec

## Purpose

Complete product-facing sampling review contracts on top of M40-M42. Sampling remains a controlled client-side model path and does not grant MCP servers local tools or model credentials.

## Scope

This slice delivers:

- sampling approval request and decision contracts
- redacted sampling review summaries
- task-augmented sampling metadata
- default-deny contracts for tool-enabled sampling loops
- redaction and permission boundary tests

Not included:

- complete approval UI
- automatic tool-enabled sampling loops
- provider-specific multimodal conversion

## Architecture

`ruse-mcp::client_sampling` owns approval/review DTOs next to existing sampling request validation. Approval requests summarize counts, token limits, task metadata, and policy flags without storing raw prompt content.

## Security

Tool-enabled sampling is denied by default even when a request includes `tools` or `toolChoice`. Approval reports and debug output do not include prompts, system prompts, tool payloads, provider payloads, model credentials, tokens, or secrets.

## Acceptance Criteria

- `cargo test -p ruse-mcp mcp_sampling_productization`
