# M47-01 HTTP Session Metadata Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure contracts for safe MCP streamable HTTP session metadata.

**Architecture:** `ruse-mcp::client_http` owns metadata parsing and exports contracts through `ruse-mcp`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M47-01-http-session-metadata-contracts.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`
- Read: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Review the spec and existing HTTP contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_session_metadata.rs`

- [x] **Step 1: Add focused metadata parser tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_session_metadata
```

Expected: tests fail before metadata contracts exist.

### Task 3: Implement Metadata Contracts

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add `McpClientHttpSessionMetadata`**
- [x] **Step 2: Add metadata parsing and validation helpers**
- [x] **Step 3: Export public contracts**
- [x] **Step 4: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_session_metadata
```

Expected: focused tests pass.

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m47 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [x] **Step 1: Update status and commit M47-01**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp http session metadata contracts"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_session_metadata` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
