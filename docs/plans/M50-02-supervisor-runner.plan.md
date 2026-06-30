# M50-02 Supervisor Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a bounded MCP HTTP stream supervisor tick runner.

**Architecture:** `agent-mcp::client_http` composes supervisor config with the M49 reconnect runner without adding scheduler ownership.

**Tech Stack:** Rust 2024 edition, async-trait, serde, fixture transports, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M50-02-supervisor-runner.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_stream_reconnect_runner.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_reconnect_trace.rs`

- [ ] **Step 1: Review runner spec and M49 reconnect tests**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_stream_supervisor_runner.rs`

- [ ] **Step 1: Add supervisor runner tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_runner
```

Expected: tests fail before supervisor runner exists.

### Task 3: Implement Runner

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add supervisor tick API**
- [ ] **Step 2: Compose M49 reconnect report**
- [ ] **Step 3: Enforce tick budget**
- [ ] **Step 4: Return typed status**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M50-02**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http stream supervisor runner"
```
