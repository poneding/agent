# M54-03 Token Exchange Runner Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the async token exchange runner and redacted diagnostics.

**Architecture:** `ruse-mcp::client_http` composes exchange DTOs, transport, response parsing, and safe reports without storage side effects.

**Tech Stack:** Rust 2024 edition, async-trait, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M54-03-token-exchange-runner-diagnostics.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`

- [x] **Step 1: Review runner diagnostic scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_oauth_token_exchange_runner.rs`

- [x] **Step 1: Add success, provider error, transport failure, invalid response, and redaction tests**

### Task 3: Implement Runner

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add diagnostic contracts**
- [x] **Step 2: Add exchange report**
- [x] **Step 3: Add async runner helper**
- [x] **Step 4: Export contracts**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_runner
CARGO_TARGET_DIR=target/codex-m54 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m54 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

Evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_runner
CARGO_TARGET_DIR=target/codex-m54 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m54 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M54-03**
