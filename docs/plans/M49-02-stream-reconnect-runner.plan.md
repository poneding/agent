# M49-02 Stream Reconnect Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a bounded reconnect runner over existing MCP HTTP stream contracts.

**Architecture:** `agent-mcp::client_http` composes stream snapshots, stream transport, and stream drain without adding background task ownership.

**Tech Stack:** Rust 2024 edition, async-trait, serde, fixture transports, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M49-02-stream-reconnect-runner.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_stream_transport.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_stream_drain.rs`

- [ ] **Step 1: Review runner spec and existing stream tests**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_stream_reconnect_runner.rs`

- [ ] **Step 1: Add reconnect runner tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_stream_reconnect_runner
```

Expected: tests fail before reconnect runner exists.

### Task 3: Implement Runner

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add reconnect runner API**
- [ ] **Step 2: Use fresh stream request snapshots per attempt**
- [ ] **Step 3: Drain accepted stream events**
- [ ] **Step 4: Stop on exhausted or non-retryable failures**
- [ ] **Step 5: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_stream_reconnect_runner
```

Expected: focused tests pass.

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m49 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [ ] **Step 1: Update status and commit M49-02**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http stream reconnect runner"
```
