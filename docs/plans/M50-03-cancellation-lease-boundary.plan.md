# M50-03 Cancellation Lease Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit cancellation and lease preflight boundaries for MCP HTTP stream supervision.

**Architecture:** `ruse-mcp::client_http` consumes lightweight control DTOs and remains independent of scheduler storage.

**Tech Stack:** Rust 2024 edition, serde, fixture transports, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M50-03-cancellation-lease-boundary.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`
- Read: `crates/ruse-scheduler/src/lib.rs`

- [x] **Step 1: Review cancellation and scheduler lease boundaries**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_stream_supervisor_cancellation.rs`

- [x] **Step 1: Add cancellation and lease tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_cancellation
```

Expected: tests fail before control contracts exist.

### Task 3: Implement Boundary

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add supervisor control contract**
- [x] **Step 2: Preflight cancellation before side effects**
- [x] **Step 3: Preflight lease validity before side effects**
- [x] **Step 4: Preserve checkpoint state on preflight stop**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M50-03**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp http stream supervisor cancellation boundary"
```

### Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_cancellation` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_runner` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
