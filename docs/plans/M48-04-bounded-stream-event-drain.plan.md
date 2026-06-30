# M48-04 Bounded Stream Event Drain Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a bounded stream event drain that updates HTTP client checkpoints safely.

**Architecture:** `McpHttpClient` records accepted stream events into existing session metadata checkpoints while keeping raw SSE data inside typed parsers.

**Tech Stack:** Rust 2024 edition, serde, SSE parsing, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M48-04-bounded-stream-event-drain.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_sse_checkpoints.rs`

- [ ] **Step 1: Review drain spec and existing checkpoint behavior**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_stream_drain.rs`

- [ ] **Step 1: Add stream drain checkpoint tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_drain
```

Expected: tests fail before stream drain API exists.

### Task 3: Implement Drain API

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add `McpClientHttpStreamDrainReport`**
- [ ] **Step 2: Add initialized stream drain method**
- [ ] **Step 3: Validate event frames and checkpoint accepted ids**
- [ ] **Step 4: Preserve redacted failures**
- [ ] **Step 5: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo test -p agent-mcp mcp_client_http_stream_drain
```

Expected: focused tests pass.

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m48 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [ ] **Step 1: Update status and commit M48-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: drain mcp http stream events"
```
