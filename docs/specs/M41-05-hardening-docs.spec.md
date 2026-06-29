# M41-05 Hardening Docs Spec

## Purpose

Document MCP sampling model bridge behavior, verify M41 focused tests, and mark M41 complete only after final quality gates pass.

## Product Context

M41 introduces model execution behind MCP sampling. Documentation must clearly distinguish safe local model bridge behavior from future approval UI, live transport dispatch, multimodal conversion, and tool-enabled sampling.

## Scope

This spec delivers:

- README documentation for the MCP sampling model bridge
- milestone status updates for M41 implementation
- final verification evidence in the M41 plan
- acceptance criteria aligned to actual focused test names

Owned by later milestones:

- CLI transport examples
- Desktop approval screenshots
- tool-enabled sampling docs

## Security and Privacy

- Docs must state that prompt and response content stay out of public diagnostics.
- Docs must state that local provider/model config overrides server hints.
- Docs must state that sampling still grants no local tool or secret permission.

## Testing Strategy

- focused M41 tests
- `git diff --check`
- workspace fmt and `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_mapping`
- `CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_provider`
- `CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_redaction`
- `CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_guardrails`
- `CARGO_TARGET_DIR=target/codex-m41 cargo xtask ci`

## Completion Signal

M41-05 is complete when docs describe the implemented model bridge boundary, all focused tests and final gates pass, and M41 is marked complete in the milestone index.
