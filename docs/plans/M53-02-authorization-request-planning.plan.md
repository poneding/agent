# M53-02 Authorization Request Planning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic OAuth authorization request planning.

**Architecture:** Pure `agent-mcp::client_http` helper over M51 metadata and M53 PKCE/state contracts.

**Tech Stack:** Rust 2024 edition, serde, URL encoding helper as needed, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M53-02-authorization-request-planning.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [x] **Step 1: Review authorization request scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_authorization_request.rs`

- [x] **Step 1: Add authorization request planning tests**

### Task 3: Implement Planning

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add input and plan DTOs**
- [x] **Step 2: Add deterministic URL construction**
- [x] **Step 3: Add validation and redacted errors**
- [x] **Step 4: Export contracts**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

### Task 5: Commit

- [x] **Step 1: Update status and commit M53-02**

Verification evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_authorization_request
CARGO_TARGET_DIR=target/codex-m53 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m53 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Result: all focused M53-02 checks passed.
