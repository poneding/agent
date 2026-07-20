# M53-03 Callback Token Exchange Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add callback validation and token exchange request DTOs.

**Architecture:** Pure `ruse-mcp::client_http` validation and DTO construction; external auth transport owns network execution.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M53-03-callback-token-exchange-contracts.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`

- [x] **Step 1: Review callback and exchange scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_oauth_callback.rs`

- [x] **Step 1: Add callback and token exchange tests**

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add callback input and validation result**
- [x] **Step 2: Add token exchange request DTO**
- [x] **Step 3: Add state and code validation**
- [x] **Step 4: Export contracts**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

### Task 5: Commit

- [x] **Step 1: Update status and commit M53-03**

Verification evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_callback
CARGO_TARGET_DIR=target/codex-m53 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m53 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

Result: all focused M53-03 checks passed.
