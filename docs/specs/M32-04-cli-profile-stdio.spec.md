# M32-04 CLI Profile Stdio Spec

## Purpose

Expose manifest-backed MCP stdio startup through `agent-cli` while preserving protocol-only stdout.

## Product Context

Users need to run an MCP stdio server from an explicit export manifest. CLI should handle path/profile arguments and leave runtime semantics in `agent-mcp`.

## Scope

This spec delivers:

- `agent-cli mcp server stdio --profile <path>` parsing
- optional `--profile-name <name>` selection
- manifest load/bootstrap errors on stderr
- subprocess tests for protocol-only stdout

Owned by other specs:

- manifest validation: M32-01
- resource/tool profile construction: M32-02 and M32-03

## Architecture

`agent-cli` reads the manifest file and calls `agent-mcp` bootstrap helpers. The running stdio loop remains unchanged.

## File Layout

```text
crates/agent-cli/src/command.rs
crates/agent-cli/tests/mcp_server_profile_command.rs
```

## Public Contracts

```bash
agent-cli mcp server stdio --profile path/to/mcp-server-exports.json --profile-name default
```

## State and Failure Handling

- invalid manifests exit non-zero before reading protocol frames
- startup errors write to stderr only
- stdout remains empty on bootstrap failure
- default stdio without `--profile` remains supported

## Security and Privacy

- redact manifest diagnostics
- never print profile resource contents
- reject `--json` for stdio mode

## Testing Strategy

- CLI parse tests
- invalid manifest subprocess test
- valid manifest subprocess test with resources/list
- stdout/stderr separation test

## Acceptance Criteria

- CLI can start stdio with a manifest profile.
- invalid startup does not emit protocol stdout.

## Completion Signal

M32-04 is complete when CLI profile stdio startup is tested end to end.
