# M49-04 Reconnect Trace Report Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed, redacted reconnect trace reports for reconnect outcomes.

**Architecture:** `agent-mcp::client_http` owns report DTOs; UI crates only consume serialized safe summaries.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M49-04-reconnect-trace-report.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Review report spec and existing redaction helpers**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_reconnect_trace.rs`

- [x] **Step 1: Add reconnect report tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_trace
```

Expected: tests fail before reconnect trace reports exist.

### Task 3: Implement Reports

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add attempt and final report contracts**
- [x] **Step 2: Add redacted failure summaries**
- [x] **Step 3: Add stable serde shape**
- [x] **Step 4: Integrate reports with runner outcomes**
- [x] **Step 5: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_trace
```

Expected: focused tests pass.

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m49 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [x] **Step 1: Update status and commit M49-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http reconnect trace reports"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_trace` passed.
- `CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_missed_event_replay` passed.
- `CARGO_TARGET_DIR=target/codex-m49 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m49 cargo clippy -p agent-mcp --all-targets -- -D warnings` passed.
