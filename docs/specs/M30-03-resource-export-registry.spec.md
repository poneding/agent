# M30-03 Resource Export Registry Spec

## Purpose

Expose selected read-only local resources through MCP server resource descriptors and bounded reads.

## Product Context

MCP clients often inspect resources before deciding which tools to call. Agent must expose only explicit, safe, read-only resources and keep secrets out of protocol output.

## Scope

This spec delivers:

- resource export descriptors
- deterministic `resources/list`
- bounded `resources/read`
- resource sensitivity and redaction policy

Owned by other specs:

- tool execution: M30-02
- stdio JSON-RPC frame handling: M30-04

## Architecture

`ruse-mcp::server_resource` owns resource descriptors and read behavior. Runtime state can later feed these descriptors, but this slice uses deterministic in-memory fixtures.

## Public Contracts

```rust
pub struct McpResourceExport {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
    pub enabled: bool,
}
```

## State and Failure Handling

- invalid URIs are rejected
- duplicate URIs are rejected
- disabled resources are hidden
- oversized reads are truncated with metadata
- missing resources return typed not-found errors

## Security and Privacy

- resources are read-only
- secret-like content is redacted from diagnostics
- sensitive resources require explicit export flags

## Testing Strategy

- stable descriptor serialization tests
- URI validation tests
- disabled/duplicate tests
- bounded read tests

## Acceptance Criteria

- `cargo test -p ruse-mcp mcp_server_resource_export` passes.
- only explicit resources are listable/readable.

## Completion Signal

M30-03 is complete when resource export contracts and fixture reads are implemented and tested.
