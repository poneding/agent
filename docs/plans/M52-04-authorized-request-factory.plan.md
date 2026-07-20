# M52-04 Authorized Request Factory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add request factory helpers that apply usable OAuth token leases to MCP HTTP requests.

**Architecture:** `ruse-mcp::client_http` composes token lease validation with M51 bearer request constructors.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M52-04-authorized-request-factory.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`

- [x] **Step 1: Review request factory scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_oauth_token_requests.rs`

- [x] **Step 1: Add authorized request factory tests**

### Task 3: Implement Factory

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add POST request factory**
- [x] **Step 2: Add GET stream request factory**
- [x] **Step 3: Add expired-token rejection**
- [x] **Step 4: Export helpers**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

### Task 5: Commit

- [x] **Step 1: Update status and commit M52-04**

Verification evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m52 cargo test -p ruse-mcp mcp_client_http_oauth_token_requests
CARGO_TARGET_DIR=target/codex-m52 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m52 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

Result: all focused M52-04 checks passed.
