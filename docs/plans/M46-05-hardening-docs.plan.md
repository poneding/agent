# M46-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document MCP HTTP SSE response behavior and complete M46 verification.

**Architecture:** Documentation hardening over runtime changes implemented in `agent-mcp`.

**Tech Stack:** Markdown docs, Cargo workspace gates.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M46-mcp-http-sse-response-runtime.spec.md`
- Read: `docs/plans/M46-mcp-http-sse-response-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M46 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M46-mcp-http-sse-response-runtime.plan.md`

- [x] **Step 1: Update README SSE response usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M46 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_sse_contracts
CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_response_mode
CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_transport
CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_sse_dispatch
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m46 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m46 cargo xtask ci
```

### Task 4: Commit

- [x] **Step 1: Commit M46-05**

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
