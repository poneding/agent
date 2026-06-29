# M43-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document MCP client stdio transport behavior and complete M43 verification.

**Architecture:** Documentation hardening over runtime changes implemented in `agent-mcp`.

**Tech Stack:** Markdown docs, Cargo workspace gates.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M43-mcp-client-stdio-transport.spec.md`
- Read: `docs/plans/M43-mcp-client-stdio-transport.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [ ] **Step 1: Read M43 specs and completed implementation slices**

Expected: scope, live transport boundary, and focused tests are clear.

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M43-mcp-client-stdio-transport.plan.md`

- [ ] **Step 1: Update README client stdio transport usage**

Add a concise note that M43 can initialize and list tools from external MCP stdio servers through local typed client contracts.

- [ ] **Step 2: Update milestone status and latest docs links**

Mark M43-01 through M43-05 complete only after implementation and verification are done.

### Task 3: Verify

- [ ] **Step 1: Run final M43 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_codec
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_session
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_process
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_mcp_client
```

Expected: focused M43 tests pass.

- [ ] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m43 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m43 cargo xtask ci
```

Expected: final gates pass.

### Task 4: Commit

- [ ] **Step 1: Commit M43-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m43 mcp client stdio transport"
```
