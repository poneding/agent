# M45-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document MCP streamable HTTP client behavior and complete M45 verification.

**Architecture:** Documentation hardening over runtime changes implemented in `agent-mcp`.

**Tech Stack:** Markdown docs, Cargo workspace gates.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M45-mcp-client-streamable-http.spec.md`
- Read: `docs/plans/M45-mcp-client-streamable-http.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M45 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M45-mcp-client-streamable-http.plan.md`

- [x] **Step 1: Update README streamable HTTP client usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M45 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_contracts
CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_mcp_client
CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_gateway_http_tool_call
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m45 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m45 cargo xtask ci
```

### Task 4: Commit

- [x] **Step 1: Commit M45-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m45 mcp client streamable http"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_contracts` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_transport` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_mcp_client` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_gateway_http_tool_call` passed.
- `git diff --check` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo xtask ci` passed.
