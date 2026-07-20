# M50-01 Stream Supervisor Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure contracts for bounded MCP HTTP stream supervision.

**Architecture:** `ruse-mcp::client_http` owns supervisor contracts and exports them through `ruse-mcp`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M50-01-stream-supervisor-contracts.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`
- Read: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Review the spec and existing reconnect contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_stream_supervisor_contracts.rs`

- [x] **Step 1: Add focused supervisor contract tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_contracts
```

Expected: tests fail before supervisor contracts exist.

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add supervisor config**
- [x] **Step 2: Add lifecycle status enum**
- [x] **Step 3: Add validation and redacted errors**
- [x] **Step 4: Export public contracts**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M50-01**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp http stream supervisor contracts"
```

### Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_contracts` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
