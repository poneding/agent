# M43-02 Client Stdio Session Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic client stdio session state over an abstract transport.

**Architecture:** `agent-mcp::client_stdio` owns session state and transport traits.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, tokio tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M43-02-client-stdio-session.spec.md`
- Read: `crates/agent-mcp/src/client_stdio.rs`
- Read: `crates/agent-mcp/src/server_session.rs`

- [ ] **Step 1: Review server session patterns and M43-01 codec**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_stdio_session.rs`

- [ ] **Step 1: Add session tests**

Add tests for matching success, mismatched ids, EOF, and summary counters.

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_session
```

### Task 3: Implement Session Runtime

**Files:**
- Modify: `crates/agent-mcp/src/client_stdio.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add transport trait and fixture transport**
- [ ] **Step 2: Add session state and summary**
- [ ] **Step 3: Add request/response matching**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m43 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m43 cargo test -p agent-mcp mcp_client_stdio_session
```

### Task 5: Commit

- [ ] **Step 1: Commit M43-02**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp client stdio session"
```
