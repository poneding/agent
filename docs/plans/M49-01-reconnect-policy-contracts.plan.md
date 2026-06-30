# M49-01 Reconnect Policy Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure contracts for bounded MCP HTTP stream reconnect policy and retry classification.

**Architecture:** `agent-mcp::client_http` owns reconnect policy contracts and exports them through `agent-mcp`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M49-01-reconnect-policy-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Review the spec and existing stream contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_reconnect_policy.rs`

- [ ] **Step 1: Add focused reconnect policy tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_policy
```

Expected: tests fail before reconnect policy contracts exist.

### Task 3: Implement Policy Contracts

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add `McpClientHttpReconnectPolicy`**
- [ ] **Step 2: Add validation and capped backoff**
- [ ] **Step 3: Add retry classification helpers**
- [ ] **Step 4: Export public contracts**
- [ ] **Step 5: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_policy
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

- [ ] **Step 1: Update status and commit M49-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http reconnect policy contracts"
```
