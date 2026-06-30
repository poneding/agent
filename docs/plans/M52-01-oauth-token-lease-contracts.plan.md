# M52-01 OAuth Token Lease Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redacted OAuth token lease contracts and token status reports.

**Architecture:** Pure DTOs and validation helpers in `agent-mcp::client_http`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M52-01-oauth-token-lease-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [x] **Step 1: Review token lease contract scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_token.rs`

- [x] **Step 1: Add token lease contract tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token
```

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add token lease DTO and validation**
- [x] **Step 2: Add token status DTO and diagnostics**
- [x] **Step 3: Add redacted debug behavior**
- [x] **Step 4: Export contracts**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m52 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m52 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M52-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http oauth token lease contracts"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token`
- `CARGO_TARGET_DIR=target/codex-m52 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m52 cargo clippy -p agent-mcp --all-targets -- -D warnings`
