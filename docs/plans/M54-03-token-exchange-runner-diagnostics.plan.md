# M54-03 Token Exchange Runner Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the async token exchange runner and redacted diagnostics.

**Architecture:** `agent-mcp::client_http` composes exchange DTOs, transport, response parsing, and safe reports without storage side effects.

**Tech Stack:** Rust 2024 edition, async-trait, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M54-03-token-exchange-runner-diagnostics.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review runner diagnostic scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_token_exchange_runner.rs`

- [ ] **Step 1: Add success, provider error, transport failure, invalid response, and redaction tests**

### Task 3: Implement Runner

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add diagnostic contracts**
- [ ] **Step 2: Add exchange report**
- [ ] **Step 3: Add async runner helper**
- [ ] **Step 4: Export contracts**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo test -p agent-mcp mcp_client_http_oauth_token_exchange_runner
CARGO_TARGET_DIR=target/codex-m54 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m54 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M54-03**
