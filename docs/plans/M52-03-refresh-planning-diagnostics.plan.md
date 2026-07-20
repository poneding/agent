# M52-03 Refresh Planning Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic refresh planning diagnostics for token leases.

**Architecture:** Pure planning helpers in `ruse-mcp::client_http`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M52-03-refresh-planning-diagnostics.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`

- [x] **Step 1: Review refresh planning scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_oauth_token_refresh.rs`

- [x] **Step 1: Add refresh planning tests**

### Task 3: Implement Planning

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add refresh plan contracts**
- [x] **Step 2: Add expiry/skew evaluation**
- [x] **Step 3: Add required-scope diagnostics**
- [x] **Step 4: Export contracts**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

### Task 5: Commit

- [x] **Step 1: Update status and commit M52-03**

Verification evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m52 cargo test -p ruse-mcp mcp_client_http_oauth_token_refresh
CARGO_TARGET_DIR=target/codex-m52 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m52 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

Result: all focused M52-03 checks passed.
