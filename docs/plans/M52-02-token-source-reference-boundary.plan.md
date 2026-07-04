# M52-02 Token Source Reference Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe token source reference contracts for external auth/storage layers.

**Architecture:** Pure DTOs in `agent-mcp::client_http`; no secret resolution.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M52-02-token-source-reference-boundary.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [x] **Step 1: Review token source reference scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_token_source.rs`

- [x] **Step 1: Add token source reference tests**

### Task 3: Implement Boundary

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add source reference DTO**
- [x] **Step 2: Add validation and redacted diagnostics**
- [x] **Step 3: Export contracts**
- [x] **Step 4: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

### Task 5: Commit

- [x] **Step 1: Update status and commit M52-02**

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_source`
- `CARGO_TARGET_DIR=target/codex-m52 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m52 cargo clippy -p agent-mcp --all-targets -- -D warnings`
