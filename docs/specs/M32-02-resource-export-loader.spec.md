# M32-02 Resource Export Loader Spec

## Purpose

Load configured MCP resource exports into `McpServerProfileRuntime` with deterministic content bounds and safe diagnostics.

## Product Context

MCP clients need explicit read-only resources. This slice turns manifest resource declarations into server resource exports while preserving policy gating at read time.

## Scope

This spec delivers:

- resource source configuration validation
- file-backed resource loading
- sensitive resource declaration handling
- profile builder integration

Owned by other specs:

- manifest serde contracts: M32-01
- CLI profile selection: M32-04

## Architecture

`ruse-mcp` owns loading from a caller-provided base directory/path context. CLI passes paths but does not inspect contents.

## File Layout

```text
crates/ruse-mcp/src/server_manifest.rs
crates/ruse-mcp/tests/mcp_server_resource_loader.rs
```

## Public Contracts

```rust
pub struct McpServerResourceExportConfig {
    pub uri: String,
    pub name: String,
    pub path: String,
    pub mime_type: Option<String>,
    pub enabled: bool,
    pub sensitive: bool,
}
```

## State and Failure Handling

- missing files fail before server startup
- paths are normalized relative to the manifest directory
- resource contents are bounded by existing registry behavior
- disabled resources are validated but hidden from list/read

## Security and Privacy

- reject secret-like paths and URIs
- do not include file contents in diagnostics
- require `sensitive=true` for secret-like resource contents

## Testing Strategy

- file load success test
- missing file failure test
- disabled resource hidden test
- sensitive content requirement test

## Acceptance Criteria

- resource configs can build a profile runtime with visible resources.

## Completion Signal

M32-02 is complete when explicit file-backed resources can be loaded into MCP server profiles.
