# M54-04 Lease Conversion Redaction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert token exchange responses into existing token leases with redaction coverage.

**Architecture:** `agent-mcp::client_http` reuses M52 token lease contracts and keeps refresh-token persistence outside the crate.

**Tech Stack:** Rust 2024 edition, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M54-04-lease-conversion-redaction.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review lease conversion scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_token_exchange_lease.rs`

- [ ] **Step 1: Add lease conversion and redaction tests**

### Task 3: Implement Lease Conversion

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add conversion helper**
- [ ] **Step 2: Reuse token lease validation**
- [ ] **Step 3: Export helper**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo test -p agent-mcp mcp_client_http_oauth_token_exchange_lease
CARGO_TARGET_DIR=target/codex-m54 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m54 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M54-04**
