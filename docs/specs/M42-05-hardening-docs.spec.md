# M42-05 Hardening Docs Spec

## Purpose

Document MCP sampling stdio dispatch behavior, verify M42 focused tests, and mark M42 complete only after final quality gates pass.

## Product Context

M42 adds protocol dispatch for sampling frames. Documentation must distinguish fixture-grade dispatch from future live process transport and approval UI.

## Scope

This spec delivers:

- README documentation for sampling stdio dispatch
- milestone status updates for M42 implementation
- final verification evidence in the M42 plan
- acceptance criteria aligned to actual focused test names

Owned by later milestones:

- live process transport examples
- Desktop approval screenshots
- tool-enabled sampling docs

## Security and Privacy

- Docs must state that error frames keep prompt/provider diagnostics redacted.
- Docs must state that sampling dispatch still grants no tools, context, or secrets.
- Docs must state that success frames contain the model response by design.

## Testing Strategy

- focused M42 tests
- `git diff --check`
- workspace fmt and `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m42 cargo test -p ruse-mcp mcp_sampling_stdio_contracts`
- `CARGO_TARGET_DIR=target/codex-m42 cargo test -p ruse-mcp mcp_sampling_stdio_dispatch`
- `CARGO_TARGET_DIR=target/codex-m42 cargo test -p ruse-mcp mcp_sampling_stdio_validation`
- `CARGO_TARGET_DIR=target/codex-m42 cargo test -p ruse-mcp mcp_sampling_stdio_redaction`
- `CARGO_TARGET_DIR=target/codex-m42 cargo xtask ci`

## Completion Signal

M42-05 is complete when docs describe implemented dispatch behavior, all focused tests and final gates pass, and M42 is marked complete in the milestone index.
