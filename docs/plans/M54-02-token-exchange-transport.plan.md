# M54-02 Token Exchange Transport Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a dedicated OAuth token exchange HTTP transport boundary.

**Architecture:** `agent-mcp::client_http` owns exchange transport contracts separate from MCP JSON-RPC transports.

**Tech Stack:** Rust 2024 edition, async-trait, reqwest, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M54-02-token-exchange-transport.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review exchange transport scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_token_exchange_transport.rs`

- [ ] **Step 1: Add fixture transport and redaction tests**

### Task 3: Implement Transport Boundary

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add HTTP exchange request/response DTOs**
- [ ] **Step 2: Add async transport trait**
- [ ] **Step 3: Add fixture transport**
- [ ] **Step 4: Add reqwest form POST implementation**
- [ ] **Step 5: Export contracts**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo test -p agent-mcp mcp_client_http_oauth_token_exchange_transport
CARGO_TARGET_DIR=target/codex-m54 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m54 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M54-02**
