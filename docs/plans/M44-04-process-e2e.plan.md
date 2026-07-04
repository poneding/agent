# M44-04 Process E2E Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add subprocess E2E coverage for live MCP stdio `tools/call`.

**Architecture:** Use `McpStdioClient::spawn` and existing gateway APIs; keep helper process test-only.

**Tech Stack:** Rust 2024 edition, tokio tests, tempfile, rustc helper process.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M44-04-process-e2e.spec.md`
- Read: `crates/agent-mcp/tests/mcp_client_stdio_process.rs`
- Read: `crates/agent-mcp/tests/mcp_client_stdio_mcp_client.rs`

- [x] **Step 1: Review existing process test helper pattern**

### Task 2: Add E2E Test

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_stdio_tool_call_process.rs`

- [x] **Step 1: Add helper process and gateway E2E test**

### Task 3: Fix Runtime Gaps

**Files:**
- Modify: `crates/agent-mcp/src/client_stdio.rs`
- Modify: `crates/agent-mcp/src/gateway.rs`

- [x] **Step 1: Fix issues revealed by process E2E**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m44 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m44 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_client_stdio_tool_call_process
```

### Task 5: Commit

- [x] **Step 1: Commit M44-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp stdio tool call process e2e"
```
