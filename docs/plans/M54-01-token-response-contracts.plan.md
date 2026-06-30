# M54-01 Token Response Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redacted token endpoint success/error response contracts.

**Architecture:** `agent-mcp::client_http` owns pure parser and validation contracts without network execution.

**Tech Stack:** Rust 2024 edition, serde_json, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M54-01-token-response-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [x] **Step 1: Review response contract scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_token_response.rs`

- [x] **Step 1: Add success, provider error, invalid JSON, and redaction tests**

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add response DTOs**
- [x] **Step 2: Add parser helpers**
- [x] **Step 3: Add redacted debug implementations**
- [x] **Step 4: Export contracts**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo test -p agent-mcp mcp_client_http_oauth_token_response
CARGO_TARGET_DIR=target/codex-m54 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m54 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo test -p agent-mcp mcp_client_http_oauth_token_response
CARGO_TARGET_DIR=target/codex-m54 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m54 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M54-01**
