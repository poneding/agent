# M41-01 Sampling Model Request Mapping Spec

## Purpose

Add a pure, deterministic mapper from safe MCP sampling request data into `agent-model::ModelRequest`.

## Product Context

M40 validates MCP sampling requests but stops at fixture results. M41-01 creates the first bridge contract without executing a model provider.

## Scope

This spec delivers:

- `agent-core::mcp_sampling` module
- `McpSamplingModelBridgeConfig`
- `McpSamplingModelBridgeIds`
- `McpSamplingModelBridgeError`
- `build_mcp_sampling_model_request`

Owned by later M41 specs:

- provider-backed execution
- stream/result conversion
- redacted provider diagnostics
- guardrails for model tool-call events

## Architecture

`agent-core` adds a normal dependency on `agent-mcp`. The mapper depends on `agent_mcp` contracts, `agent_model::ModelRequest`, and `agent_types` IDs.

## Public Contracts

```rust
pub fn build_mcp_sampling_model_request(
    request: &McpCreateMessageRequestParams,
    config: &McpSamplingModelBridgeConfig,
    ids: McpSamplingModelBridgeIds,
) -> Result<ModelRequest, McpSamplingModelBridgeError>;
```

## State and Failure Handling

- The mapper is pure and has no side effects.
- It rejects empty provider/model config.
- It validates the MCP request with the M40 default policy before mapping.
- It maps only text content.
- It rejects image, audio, tool-use, and tool-result content without leaking raw content.

## Security and Privacy

- Error messages include stable codes and safe field names only.
- Server model preferences do not override local provider/model config.

## Testing Strategy

- builds a model request with local provider/model and stable IDs
- includes system prompt and role-prefixed text messages
- ignores server model hints for provider/model selection
- rejects unsupported media without prompt leaks

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_mapping`
- `CARGO_TARGET_DIR=target/codex-m41 cargo clippy -p agent-core --all-targets -- -D warnings`

## Completion Signal

M41-01 is complete when mapping tests pass and public contracts are exported from `agent-core`.
