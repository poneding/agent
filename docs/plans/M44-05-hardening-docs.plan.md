# M44-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document MCP stdio tool-call behavior and complete M44 verification.

**Architecture:** Documentation hardening over runtime changes implemented in `ruse-mcp`.

**Tech Stack:** Markdown docs, Cargo workspace gates.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M44-mcp-client-stdio-tool-call.spec.md`
- Read: `docs/plans/M44-mcp-client-stdio-tool-call.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M44 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M44-mcp-client-stdio-tool-call.plan.md`

- [x] **Step 1: Update README live stdio tool-call usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M44 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m44 cargo test -p ruse-mcp mcp_client_stdio_tool_call
CARGO_TARGET_DIR=target/codex-m44 cargo test -p ruse-mcp mcp_gateway_stdio_tool_call
CARGO_TARGET_DIR=target/codex-m44 cargo test -p ruse-mcp mcp_client_stdio_tool_call_process
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m44 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m44 cargo xtask ci
```

### Task 4: Commit

- [x] **Step 1: Commit M44-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m44 mcp client stdio tool call"
```
