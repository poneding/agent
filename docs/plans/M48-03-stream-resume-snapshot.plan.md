# M48-03 Stream Resume Snapshot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose initialized HTTP client stream request snapshots that include safe resume headers.

**Architecture:** `McpHttpClient` builds typed stream request snapshots from initialized state without mutating session metadata.

**Tech Stack:** Rust 2024 edition, async-trait, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M48-03-stream-resume-snapshot.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_session_headers.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_sse_checkpoints.rs`

- [x] **Step 1: Review resume snapshot spec and existing session tests**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_stream_resume.rs`

- [x] **Step 1: Add initialized stream snapshot tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_resume
```

Expected: tests fail before snapshot API exists.

### Task 3: Implement Snapshot API

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add stream request snapshot method**
- [x] **Step 2: Require initialized matching server id**
- [x] **Step 3: Include safe session and checkpoint headers**
- [x] **Step 4: Prove snapshot does not mutate state**
- [x] **Step 5: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_resume
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

- [x] **Step 1: Update status and commit M48-03**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http stream resume snapshots"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_resume` passed.
- `CARGO_TARGET_DIR=target/codex-m48 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m48 cargo clippy -p agent-mcp --all-targets -- -D warnings` passed.
