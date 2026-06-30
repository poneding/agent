# M50-04 Supervisor Reports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed, redacted MCP HTTP stream supervisor reports.

**Architecture:** `agent-mcp::client_http` owns report DTOs; UI crates consume serialized safe summaries.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M50-04-supervisor-reports.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Review report spec and existing reconnect report helpers**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_stream_supervisor_reports.rs`

- [ ] **Step 1: Add supervisor report tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_reports
```

Expected: tests fail before supervisor reports exist.

### Task 3: Implement Reports

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add supervisor report contract**
- [ ] **Step 2: Add redacted terminal summaries**
- [ ] **Step 3: Add stable serde shape**
- [ ] **Step 4: Integrate reports with runner outcomes**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M50-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http stream supervisor reports"
```
