# M43-03 Client Stdio Process Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add spawned process stdio transport for MCP client sessions.

**Architecture:** `ruse-mcp` owns process IO behind the M43 transport trait.

**Tech Stack:** Rust 2024 edition, Tokio process/IO, serde JSON, ruse-mcp tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M43-03-client-stdio-process.spec.md`
- Read: `crates/ruse-mcp/src/client_stdio.rs`
- Read: `crates/ruse-mcp/src/config.rs`

- [x] **Step 1: Review stdio config and session transport trait**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_stdio_process.rs`

- [x] **Step 1: Add process transport tests**

Add tests for fixture process exchange, disabled config, early exit, and stderr redaction.

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_process
```

### Task 3: Implement Process Transport

**Files:**
- Modify: `crates/ruse-mcp/Cargo.toml`
- Modify: `crates/ruse-mcp/src/client_stdio.rs`

- [x] **Step 1: Enable Tokio process/IO features if needed**
- [x] **Step 2: Add spawn, send, receive, and close behavior**
- [x] **Step 3: Add redacted process diagnostics**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m43 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_process
```

### Task 5: Commit

- [x] **Step 1: Commit M43-03**

Run:

```bash
git add crates/ruse-mcp docs Cargo.lock
git commit -m "feat: add mcp client stdio process transport"
```
