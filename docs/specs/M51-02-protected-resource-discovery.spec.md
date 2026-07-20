# M51-02 Protected Resource Discovery Spec

## Purpose

Add typed protected-resource discovery contracts and parsers for MCP HTTP authorization metadata.

## Product Context

M51-01 defines metadata contracts. The runtime also needs a safe way to consume discovery documents from protected resource metadata and authorization server metadata endpoints.

## Scope

This spec delivers:

- parser functions from JSON values into M51 metadata DTOs
- bounded field validation and unknown-field tolerance
- discovery diagnostics that do not expose raw response bodies

Owned by other specs:

- policy-level authorization diagnostics in M51-03
- live network discovery transport beyond fixture parsing

## Architecture

Parsing remains in `ruse-mcp::client_http` and accepts caller-provided JSON values. This slice does not introduce live network fetching.

## Public Contracts

```rust
pub fn parse_mcp_client_http_oauth_metadata_json(value: &serde_json::Value)
    -> Result<McpClientHttpOAuthMetadata, McpError>;
```

## State and Failure Handling

- Missing optional fields are accepted.
- Malformed endpoint values return redacted invalid-config errors.
- Unknown fields are ignored.

## Security and Privacy

- Raw metadata JSON is never echoed in errors.
- Discovery diagnostics expose field presence and validation state only.

## Testing Strategy

- parse valid metadata
- reject invalid endpoint shapes
- tolerate unknown fields
- redaction tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_discovery`
- `CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p ruse-mcp --all-targets -- -D warnings`

## Completion Signal

M51-02 is complete when discovery JSON can be parsed into typed metadata with redacted validation behavior.
