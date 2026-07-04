# M40-05 Hardening Docs Spec

## Purpose

Document MCP sampling behavior, verify M40 focused tests, and mark the milestone complete only after final quality gates pass.

## Product Context

M40 adds a sensitive nested model-call primitive. README and milestone docs must make the safety boundary clear: sampling requests are model metadata and prompt payloads, not permission grants or local tool access.

## Scope

This spec delivers:

- README documentation for MCP sampling safe subset and example usage
- milestone status updates for M40 implementation
- final verification evidence in the M40 plan
- acceptance criteria aligned to the actual focused test names

Owned by later milestones:

- live model sampling docs
- Desktop approval screenshots or UX docs
- tool-enabled sampling docs

## Architecture

Documentation updates live in `README.md`, `docs/milestones/README.md`, and M40 plan/spec files. Runtime behavior remains in crates.

## State and Failure Handling

- Docs must state that M40 does not provide live model invocation, tool-enabled sampling, or approval UI.
- If final gates fail, M40 remains in progress and the failing check is recorded in the plan before fixes continue.
- Planning lists must not point the next implementation slice at already completed work.

## Security and Privacy

- README examples must not include real prompts with secrets or model credentials.
- Docs must explicitly say prompt and response content are not emitted in gateway reports.
- Sampling does not grant filesystem, command, Git, browser, Desktop, or secret access.

## Testing Strategy

- focused M40 tests
- `git diff --check`
- workspace fmt/clippy/test gates through `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_contracts`
- `CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_validation`
- `CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_client_session_sampling`
- `CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_gateway_sampling`
- `CARGO_TARGET_DIR=target/codex-m40 cargo xtask ci`

## Completion Signal

M40-05 is complete when docs describe the implemented sampling boundary, all focused tests and final gates pass, and M40 is marked complete in the milestone index.
