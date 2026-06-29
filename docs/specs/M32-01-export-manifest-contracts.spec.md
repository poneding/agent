# M32-01 Export Manifest Contracts Spec

## Purpose

Add versioned MCP server export manifest contracts so profile configuration can be parsed, validated, selected, and inspected before server startup.

## Product Context

M32 needs a typed source of truth for explicit exports. This slice creates the durable contract without yet loading file contents or starting CLI stdio from the manifest.

## Scope

This spec delivers:

- schema version constant
- manifest/profile/tool/resource config structs
- JSON parsing helper
- profile selection helper
- validation and redacted diagnostics

Owned by other specs:

- resource file content loading: M32-02
- tool catalog binding: M32-03
- CLI flags: M32-04

## Architecture

`agent-mcp::server_manifest` owns the serde contracts. It may reference existing server profile validation rules but does not execute tools or read resource files.

## File Layout

```text
crates/agent-mcp/src/server_manifest.rs
crates/agent-mcp/src/lib.rs
crates/agent-mcp/tests/mcp_server_manifest.rs
```

## Public Contracts

```rust
pub const MCP_SERVER_EXPORT_MANIFEST_SCHEMA_VERSION: u32 = 1;

pub fn parse_mcp_server_export_manifest_json(
    input: &str,
) -> Result<McpServerExportManifest, McpError>;
```

## State and Failure Handling

- unsupported schema versions fail
- empty manifests fail
- duplicate profile names, tool names, and resource URIs fail
- disabled entries may be present but are excluded from enabled summary fields later
- unknown profile selection fails with a redacted message

## Security and Privacy

- reject secret-like names, URIs, paths, and descriptions
- never echo raw JSON on parse errors
- bound all diagnostic text

## Testing Strategy

- stable serde shape test
- parse/validate success test
- duplicate rejection test
- secret-like rejection test
- profile selection failure test

## Acceptance Criteria

- `cargo test -p agent-mcp mcp_server_manifest` passes.
- contracts are exported from `agent_mcp`.

## Completion Signal

M32-01 is complete when manifest contracts are available and independently validated.
