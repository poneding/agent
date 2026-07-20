# M34-03 Preflight Diagnostics Spec

## Purpose

Improve inspection diagnostics so profile startup risks are visible before stdio starts.

## Scope

This spec delivers diagnostics for:

- missing workspace for tools
- invalid workspace root
- unknown tool declarations
- missing resource files
- disabled entries hidden from active exports

## Architecture

Diagnostics remain in `ruse-mcp`; CLI renders them.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-mcp mcp_server_profile_preflight_diagnostics`
- `CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-cli mcp_server_profile_inspect_command`

## Completion Signal

M34-03 is complete when preflight diagnostics match startup behavior and remain redacted.
