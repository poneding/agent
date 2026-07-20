# M48-02 Reqwest GET SSE Stream Open Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a live reqwest-backed GET stream-open boundary for MCP streamable HTTP SSE streams.

**Architecture:** `ruse-mcp::client_http` adds stream transport contracts next to the existing POST transport while keeping raw headers and stream bodies inside the transport boundary.

**Tech Stack:** Rust 2024 edition, async-trait, reqwest, serde, SSE parsing, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M48-02-reqwest-get-sse-stream-open.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`
- Read: `crates/ruse-mcp/tests/mcp_client_http_transport.rs`

- [x] **Step 1: Review stream-open spec and existing reqwest tests**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_stream_transport.rs`

- [x] **Step 1: Add fixture and reqwest GET SSE tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p ruse-mcp mcp_client_http_stream_transport
```

Expected: tests fail before stream transport contracts exist.

### Task 3: Implement Stream Transport

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add `McpClientHttpStreamOpen`**
- [x] **Step 2: Add `McpClientHttpStreamTransport` and fixture transport**
- [x] **Step 3: Implement reqwest GET SSE stream open**
- [x] **Step 4: Preserve redacted diagnostics**
- [x] **Step 5: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p ruse-mcp mcp_client_http_stream_transport
```

Expected: focused tests pass.

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m48 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [x] **Step 1: Update status and commit M48-02**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: open mcp http get sse streams"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m48 cargo test -p ruse-mcp mcp_client_http_stream_transport` passed.
- `CARGO_TARGET_DIR=target/codex-m48 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m48 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
