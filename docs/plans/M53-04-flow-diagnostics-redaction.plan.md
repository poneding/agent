# M53-04 Flow Diagnostics Redaction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add serializable, redacted OAuth flow diagnostics.

**Architecture:** `ruse-mcp::client_http` diagnostic contracts reused by M53 flow helpers.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M53-04-flow-diagnostics-redaction.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`

- [x] **Step 1: Review diagnostic scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_oauth_flow_diagnostics.rs`

- [x] **Step 1: Add flow diagnostic tests**

### Task 3: Implement Diagnostics

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add diagnostic severity/code contracts**
- [x] **Step 2: Add redacted builder helpers**
- [x] **Step 3: Wire diagnostics into flow helpers**
- [x] **Step 4: Export contracts**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m53 cargo fmt --all
CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_flow_diagnostics
CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_callback
CARGO_TARGET_DIR=target/codex-m53 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m53 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M53-04**
