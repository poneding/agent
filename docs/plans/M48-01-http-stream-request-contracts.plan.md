# M48-01 HTTP Stream Request Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure contracts for safe MCP streamable HTTP GET stream requests.

**Architecture:** `agent-mcp::client_http` owns stream request construction and exports contracts through `agent-mcp`.

**Tech Stack:** Rust 2024 edition, serde, HTTP headers, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M48-01-http-stream-request-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Review the spec and existing HTTP request contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_stream_request.rs`

- [x] **Step 1: Add focused stream request tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_request
```

Expected: tests fail before stream request contracts exist.

### Task 3: Implement Stream Request Contracts

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add `McpClientHttpStreamRequest`**
- [x] **Step 2: Add stream request construction from config and session metadata**
- [x] **Step 3: Add `Last-Event-ID` conflict handling**
- [x] **Step 4: Export public contracts**
- [x] **Step 5: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_request
```

Expected: focused tests pass.

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m48 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [x] **Step 1: Update status and commit M48-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http stream request contracts"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_request` passed.
- `CARGO_TARGET_DIR=target/codex-m48 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m48 cargo clippy -p agent-mcp --all-targets -- -D warnings` passed.
