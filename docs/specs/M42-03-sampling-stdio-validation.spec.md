# M42-03 Sampling Stdio Validation Spec

## Purpose

Prove sampling stdio dispatch rejects invalid sampling requests before provider execution.

## Product Context

M42 dispatches untrusted JSON-RPC frames from an MCP server. Validation-before-provider behavior keeps sampling safe and deterministic.

## Scope

This spec delivers:

- call-count tests for validation-before-provider behavior
- explicit JSON-RPC failures for unsupported tools/context/media requests
- disabled and malformed request handling aligned with M40/M41 safe subset

Owned by later specs:

- redaction-specific message hardening
- live process transport recovery

## Architecture

Validation stays centralized in `create_mcp_sampling_message` and `validate_mcp_sampling_request`. M42 dispatcher tests prove the provider is not called when validation rejects.

## Public Contracts

No new public contracts are required beyond M42-02 dispatcher behavior.

## State and Failure Handling

- Invalid sampling requests return JSON-RPC failure.
- Provider call count remains zero for invalid requests.
- Error code and message remain stable enough for CLI/Desktop diagnostics.

## Security and Privacy

- Rejected request errors do not echo tool JSON, media data, prompt text, or context details.

## Testing Strategy

- reject `tools` before provider
- reject `includeContext` before provider
- reject invalid token limits before provider
- reject malformed params before provider

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_validation`

## Completion Signal

M42-03 is complete when invalid sampling stdio frames are rejected deterministically before provider execution.
