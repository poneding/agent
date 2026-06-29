# M34-04 JSON Redaction Regressions Spec

## Purpose

Lock profile inspection JSON and diagnostics against secret leakage and unstable output.

## Scope

This spec delivers:

- JSON snapshot-style tests
- secret-like manifest value regression tests
- protocol-only stdio startup regression tests
- text/JSON consistency checks

## Architecture

`agent-mcp` redacts diagnostics; `agent-cli` preserves redacted output.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-cli mcp_server_profile_inspect_command`
- `CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-mcp mcp_server_profile_inspection`

## Completion Signal

M34-04 is complete when inspection output is stable, parseable, and redacted.
