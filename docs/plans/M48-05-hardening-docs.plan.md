# M48-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document and verify the completed M48 MCP HTTP stream ownership foundation.

**Architecture:** Documentation-only slice after runtime implementation. README and milestone status must reflect the exact implemented boundary.

**Tech Stack:** Markdown, Cargo fmt, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M48-mcp-http-stream-ownership-runtime.spec.md`
- Read: `docs/plans/M48-mcp-http-stream-ownership-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [ ] **Step 1: Read M48 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M48-mcp-http-stream-ownership-runtime.plan.md`

- [ ] **Step 1: Update README stream ownership usage**
- [ ] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [ ] **Step 1: Run final M48 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_request
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_transport
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_resume
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_drain
```

- [ ] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m48 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m48 cargo xtask ci
```

### Task 4: Commit

- [ ] **Step 1: Commit M48-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m48 mcp http stream ownership runtime"
```
