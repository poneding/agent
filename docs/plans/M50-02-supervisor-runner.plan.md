# M50-02 Supervisor Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a bounded MCP HTTP stream supervisor tick runner.

**Architecture:** `ruse-mcp::client_http` composes supervisor config with the M49 reconnect runner without adding scheduler ownership.

**Tech Stack:** Rust 2024 edition, async-trait, serde, fixture transports, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M50-02-supervisor-runner.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`
- Read: `crates/ruse-mcp/tests/mcp_client_http_stream_reconnect_runner.rs`
- Read: `crates/ruse-mcp/tests/mcp_client_http_reconnect_trace.rs`

- [x] **Step 1: Review runner spec and M49 reconnect tests**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_stream_supervisor_runner.rs`

- [x] **Step 1: Add supervisor runner tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_runner
```

Expected: tests fail before supervisor runner exists.

### Task 3: Implement Runner

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add supervisor tick API**
- [x] **Step 2: Compose M49 reconnect report**
- [x] **Step 3: Enforce tick budget**
- [x] **Step 4: Return typed status**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M50-02**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp http stream supervisor runner"
```

### Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_runner` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
