# M34-02 CLI Profile Inspect Command Spec

## Purpose

Expose profile inspection through `ruse-cli`.

## Product Context

Users and scripts need a command that validates and summarizes an export profile before starting MCP stdio.

## Scope

This spec delivers:

- `ruse-cli mcp server profile inspect`
- text rendering
- JSON rendering under `--json`
- parser tests and safe failure behavior

Owned by other specs:

- ruse-mcp inspection contracts
- extended diagnostics and redaction regressions

## Architecture

CLI reads manifest files, calls `ruse-mcp` inspection, and renders typed output. It must not duplicate profile validation logic.

## Public Contracts

```bash
ruse-cli mcp server profile inspect --profile exports.json
ruse-cli mcp server profile inspect --profile exports.json --profile-name default --workspace .
ruse-cli --json mcp server profile inspect --profile exports.json --workspace .
```

## State and Failure Handling

- Invalid JSON exits non-zero with no secret leakage.
- Inspection diagnostics do not force non-zero exit unless manifest parsing/profile selection fails.
- JSON output is a single parseable object or frame matching existing CLI style.

## Security and Privacy

- No resource contents on stdout.
- stderr errors are redacted.

## Testing Strategy

- parser tests
- text output tests
- JSON parseability test
- invalid manifest failure test

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-cli mcp_server_profile_inspect_command`

## Completion Signal

M34-02 is complete when CLI can inspect profile manifests in text and JSON.
