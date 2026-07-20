# M32-03 Tool Export Catalog Spec

## Purpose

Bind manifest tool declarations to a controlled catalog of exportable local tools.

## Product Context

MCP tool export must remain explicit and policy-gated. This slice avoids arbitrary tool registration from manifests by requiring a known catalog.

## Scope

This spec delivers:

- exportable tool catalog contract
- manifest tool declaration validation against the catalog
- profile builder integration for enabled/disabled tools
- safe unknown-tool diagnostics

Owned by other specs:

- concrete local tool implementations: prior tool milestones
- CLI selection: M32-04

## Architecture

`ruse-mcp` owns generic catalog matching. Callers provide the catalog; the manifest never instantiates arbitrary code.

## File Layout

```text
crates/ruse-mcp/src/server_manifest.rs
crates/ruse-mcp/tests/mcp_server_tool_catalog.rs
```

## Public Contracts

```rust
pub struct McpServerToolExportConfig {
    pub name: String,
    pub enabled: bool,
}
```

## State and Failure Handling

- unknown tools fail before startup
- duplicates fail during manifest validation
- disabled tools are registered as disabled and hidden from list/call

## Security and Privacy

- reject secret-like tool names
- tool calls still pass through policy
- tool inputs are not stored in profile summaries

## Testing Strategy

- catalog match success test
- unknown tool failure test
- denied call still does not execute test
- disabled tool hidden test

## Acceptance Criteria

- manifest tool declarations can construct a profile from a caller-provided catalog.

## Completion Signal

M32-03 is complete when explicit manifest tools can be registered from a safe catalog.
