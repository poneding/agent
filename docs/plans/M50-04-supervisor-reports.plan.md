# M50-04 Supervisor Reports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed, redacted MCP HTTP stream supervisor reports.

**Architecture:** `ruse-mcp::client_http` owns report DTOs; UI crates consume serialized safe summaries.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M50-04-supervisor-reports.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`
- Read: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Review report spec and existing reconnect report helpers**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_stream_supervisor_reports.rs`

- [x] **Step 1: Add supervisor report tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_reports
```

Expected: tests fail before supervisor reports exist.

### Task 3: Implement Reports

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add supervisor report contract**
- [x] **Step 2: Add redacted terminal summaries**
- [x] **Step 3: Add stable serde shape**
- [x] **Step 4: Integrate reports with runner outcomes**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M50-04**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp http stream supervisor reports"
```

### Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_reports` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
