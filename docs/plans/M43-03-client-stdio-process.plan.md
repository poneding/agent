# M43-03 Client Stdio Process Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add spawned process stdio transport for MCP client sessions.

**Architecture:** `agent-mcp` owns process IO behind the M43 transport trait.

**Tech Stack:** Rust 2024 edition, Tokio process/IO, serde JSON, agent-mcp tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M43-03-client-stdio-process.spec.md`
- Read: `crates/agent-mcp/src/client_stdio.rs`
- Read: `crates/agent-mcp/src/config.rs`

- [ ] **Step 1: Review stdio config and session transport trait**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_stdio_process.rs`

- [ ] **Step 1: Add process transport tests**

Add tests for fixture process exchange, disabled config, early exit, and stderr redaction.

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_process
```

### Task 3: Implement Process Transport

**Files:**
- Modify: `crates/agent-mcp/Cargo.toml`
- Modify: `crates/agent-mcp/src/client_stdio.rs`

- [ ] **Step 1: Enable Tokio process/IO features if needed**
- [ ] **Step 2: Add spawn, send, receive, and close behavior**
- [ ] **Step 3: Add redacted process diagnostics**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m43 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_process
```

### Task 5: Commit

- [ ] **Step 1: Commit M43-03**

Run:

```bash
git add crates/agent-mcp docs Cargo.lock
git commit -m "feat: add mcp client stdio process transport"
```
