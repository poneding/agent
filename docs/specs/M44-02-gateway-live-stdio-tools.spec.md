# M44-02 Gateway Live Stdio Tools Spec

## Purpose

Prove that the existing `McpGateway` can discover, register, and execute tools through a live stdio-backed `McpStdioClient`.

## Product Context

M44-01 adds typed `tools/call` mapping. This slice verifies the local gateway path uses that mapping without requiring UI code or alternate MCP protocol logic.

## Scope

This spec delivers:

- gateway tests with `McpStdioClient` fixture transport
- local tool registry execution through `McpGatewayTool`
- bounded output verification through the existing gateway limit

Owned by other specs:

- spawned process E2E
- final docs

## Architecture

`McpGateway` remains unchanged unless tests reveal a bug. `McpStdioClient` satisfies the existing `McpGatewayClient` trait.

## Public Contracts

- `McpGateway::discover_tools`
- `McpGateway::register_tools`
- `Tool::execute` over a registered MCP gateway tool

## State and Failure Handling

- Discovery performs initialize and tools/list.
- Registered tool execution sends one `tools/call`.
- Output is bounded by `McpGateway::with_max_response_bytes`.

## Security and Privacy

- Gateway redacts secret-like local input before passing it into `call_tool`.
- Local policy remains the gate before tool execution in higher-level runtime paths.

## Testing Strategy

- fixture-backed `McpStdioClient` discovery and execution
- output bounding assertion
- redacted argument assertion through sent frame inspection

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m44 cargo test -p ruse-mcp mcp_gateway_stdio_tool_call`

## Completion Signal

M44-02 is complete when gateway-registered local tools can execute through a stdio client fixture.
