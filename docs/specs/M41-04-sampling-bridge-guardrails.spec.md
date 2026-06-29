# M41-04 Sampling Bridge Guardrails Spec

## Purpose

Add explicit guardrails for unsupported MCP sampling paths before broader transport and UI integration.

## Product Context

M41 can execute safe text-only sampling. Guardrails make unsupported model stream tool calls, context inclusion, server-requested tools, and media conversion fail predictably.

## Scope

This spec delivers:

- rejection of model `ToolCall` stream events
- explicit unsupported media conversion errors
- validation-before-provider tests for tools/context
- bridge call count or fixture tests proving invalid requests do not stream

Owned by later specs:

- tool-enabled sampling loops
- multimodal conversion
- approval UI

## Architecture

Guardrails live in `agent-core::mcp_sampling` around mapping and stream collection. They reuse `agent-mcp` validation and bridge diagnostics.

## State and Failure Handling

- Invalid MCP requests fail before model provider execution.
- ToolCall stream events fail after stream collection with redacted diagnostics.
- Guardrail failures do not produce partial MCP results.

## Security and Privacy

- No tool input JSON from rejected ToolCall events appears in public errors.
- Rejected context inclusion does not expose context values.

## Testing Strategy

- tool request validation fails before provider stream
- context inclusion validation fails before provider stream
- model tool-call stream fails without tool input leak
- media content rejection remains deterministic

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_guardrails`

## Completion Signal

M41-04 is complete when unsupported sampling paths fail deterministically and safely under focused tests.
