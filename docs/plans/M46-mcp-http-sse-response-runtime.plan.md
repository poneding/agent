# M46 MCP HTTP SSE Response Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe, typed response-stream foundation for MCP streamable HTTP clients.

**Architecture:** `agent-mcp` owns SSE parsing, response-mode contracts, reqwest response classification, client dispatch, redaction, and tests. CLI/Desktop remain consumers of typed runtime contracts.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, reqwest, HTTP/SSE parsing, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M46 Technical Design

**Files:**
- Create: `docs/specs/M46-00-mcp-http-sse-response-technical-design.spec.md`
- Create: `docs/plans/M46-00-mcp-http-sse-response-technical-design.plan.md`

- [x] **Step 1: Write the M46 technical design baseline**

Define crate ownership, SSE event contracts, response-mode boundaries, stream bounds, redaction behavior, and verification gates.

### Task 2: Decompose M46 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M46-01-sse-event-contracts.spec.md`
- Create: `docs/plans/M46-01-sse-event-contracts.plan.md`
- Create: `docs/specs/M46-02-http-response-mode.spec.md`
- Create: `docs/plans/M46-02-http-response-mode.plan.md`
- Create: `docs/specs/M46-03-reqwest-sse-transport.spec.md`
- Create: `docs/plans/M46-03-reqwest-sse-transport.plan.md`
- Create: `docs/specs/M46-04-http-client-sse-dispatch.spec.md`
- Create: `docs/plans/M46-04-http-client-sse-dispatch.plan.md`
- Create: `docs/specs/M46-05-hardening-docs.spec.md`
- Create: `docs/plans/M46-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M46 Slices in Order

**Files:**
- Execute: `docs/plans/M46-01-sse-event-contracts.plan.md`
- Execute: `docs/plans/M46-02-http-response-mode.plan.md`
- Execute: `docs/plans/M46-03-reqwest-sse-transport.plan.md`
- Execute: `docs/plans/M46-04-http-client-sse-dispatch.plan.md`
- Execute: `docs/plans/M46-05-hardening-docs.plan.md`

- [x] **Step 1: Add SSE event parsing contracts**
- [x] **Step 2: Add HTTP response-mode contracts**
- [x] **Step 3: Add reqwest `text/event-stream` response handling**
- [x] **Step 4: Dispatch HTTP client calls from SSE response frames**
- [x] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M46

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M46-mcp-http-sse-response-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m46 cargo xtask ci
```

Expected: all checks pass.

- [x] **Step 2: Mark M46 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m46 mcp http sse response runtime"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_sse_contracts` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_response_mode` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_transport` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_sse_dispatch` passed.
- `git diff --check` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo xtask ci` passed.
