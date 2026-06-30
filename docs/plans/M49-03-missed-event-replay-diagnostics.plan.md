# M49-03 Missed Event Replay Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe replay diagnostics proving reconnect attempts requested resume from local checkpoints.

**Architecture:** `agent-mcp::client_http` derives diagnostics from stream snapshots, checkpoint state, and drain reports without mutating state.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M49-03-missed-event-replay-diagnostics.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_stream_resume.rs`

- [ ] **Step 1: Review replay diagnostics spec and snapshot behavior**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_missed_event_replay.rs`

- [ ] **Step 1: Add replay diagnostics tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_missed_event_replay
```

Expected: tests fail before replay diagnostics exist.

### Task 3: Implement Diagnostics

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add replay diagnostic contract**
- [ ] **Step 2: Detect requested resume without raw event id exposure**
- [ ] **Step 3: Detect checkpoint advancement**
- [ ] **Step 4: Integrate diagnostics with reconnect outcomes**
- [ ] **Step 5: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_missed_event_replay
```

Expected: focused tests pass.

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m49 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [ ] **Step 1: Update status and commit M49-03**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http missed event replay diagnostics"
```
