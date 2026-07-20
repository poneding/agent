# M47 MCP HTTP Session Resume Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the session metadata and event checkpoint foundation for resumable MCP streamable HTTP clients.

**Architecture:** `ruse-mcp` owns HTTP session metadata, response envelopes, session-aware headers, SSE checkpoints, redaction, and tests. CLI/Desktop remain consumers of typed runtime contracts.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, reqwest, HTTP headers, SSE parsing, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M47 Technical Design

**Files:**
- Create: `docs/specs/M47-00-mcp-http-session-resume-technical-design.spec.md`
- Create: `docs/plans/M47-00-mcp-http-session-resume-technical-design.plan.md`

- [x] **Step 1: Write the M47 technical design baseline**

Define crate ownership, metadata contracts, header propagation, checkpoint state, redaction behavior, and verification gates.

### Task 2: Decompose M47 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M47-01-http-session-metadata-contracts.spec.md`
- Create: `docs/plans/M47-01-http-session-metadata-contracts.plan.md`
- Create: `docs/specs/M47-02-http-response-envelope.spec.md`
- Create: `docs/plans/M47-02-http-response-envelope.plan.md`
- Create: `docs/specs/M47-03-session-header-propagation.spec.md`
- Create: `docs/plans/M47-03-session-header-propagation.plan.md`
- Create: `docs/specs/M47-04-sse-checkpoint-tracking.spec.md`
- Create: `docs/plans/M47-04-sse-checkpoint-tracking.plan.md`
- Create: `docs/specs/M47-05-hardening-docs.spec.md`
- Create: `docs/plans/M47-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M47 Slices in Order

**Files:**
- Execute: `docs/plans/M47-01-http-session-metadata-contracts.plan.md`
- Execute: `docs/plans/M47-02-http-response-envelope.plan.md`
- Execute: `docs/plans/M47-03-session-header-propagation.plan.md`
- Execute: `docs/plans/M47-04-sse-checkpoint-tracking.plan.md`
- Execute: `docs/plans/M47-05-hardening-docs.plan.md`

- [x] **Step 1: Add HTTP session metadata contracts**
- [x] **Step 2: Add HTTP response envelope metadata capture**
- [x] **Step 3: Propagate session headers after initialize**
- [x] **Step 4: Track SSE last-event checkpoints**
- [x] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M47

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M47-mcp-http-session-resume-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo xtask ci
```

Expected: all checks pass.

- [x] **Step 2: Mark M47 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m47 mcp http session resume runtime"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_session_metadata` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_response_envelope` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_session_headers` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_sse_checkpoints` passed.
- `git diff --check` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo xtask ci` passed. The `definitely-not-a-real-rustc-flag` output is expected from an existing quality-gate fixture.
