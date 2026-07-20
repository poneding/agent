# M46-01 SSE Event Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure SSE event parsing contracts for MCP HTTP client responses.

**Architecture:** `ruse-mcp::client_http` owns parser contracts and exports them through `ruse-mcp`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M46-01-sse-event-contracts.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`
- Read: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Review the spec and existing HTTP contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_sse_contracts.rs`

- [x] **Step 1: Add focused SSE parser tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_sse_contracts
```

Expected: tests fail before the parser exists.

### Task 3: Implement Parser

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add `McpClientHttpSseEvent` and parsing function**
- [x] **Step 2: Export public contracts**
- [x] **Step 3: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_sse_contracts
```

Expected: focused tests pass.

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m46 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m46 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [x] **Step 1: Update status and commit M46-01**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp http sse event contracts"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_sse_contracts` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
