# M34-02 CLI Profile Inspect Command Spec

## Purpose

Expose profile inspection through `agent-cli`.

## Product Context

Users and scripts need a command that validates and summarizes an export profile before starting MCP stdio.

## Scope

This spec delivers:

- `agent-cli mcp server profile inspect`
- text rendering
- JSON rendering under `--json`
- parser tests and safe failure behavior

Owned by other specs:

- agent-mcp inspection contracts
- extended diagnostics and redaction regressions

## Architecture

CLI reads manifest files, calls `agent-mcp` inspection, and renders typed output. It must not duplicate profile validation logic.

## Public Contracts

```bash
agent-cli mcp server profile inspect --profile exports.json
agent-cli mcp server profile inspect --profile exports.json --profile-name default --workspace .
agent-cli --json mcp server profile inspect --profile exports.json --workspace .
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

- `CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-cli mcp_server_profile_inspect_command`

## Completion Signal

M34-02 is complete when CLI can inspect profile manifests in text and JSON.
