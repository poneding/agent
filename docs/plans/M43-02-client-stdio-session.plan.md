# M43-02 Client Stdio Session Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic client stdio session state over an abstract transport.

**Architecture:** `ruse-mcp::client_stdio` owns session state and transport traits.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, tokio tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M43-02-client-stdio-session.spec.md`
- Read: `crates/ruse-mcp/src/client_stdio.rs`
- Read: `crates/ruse-mcp/src/server_session.rs`

- [x] **Step 1: Review server session patterns and M43-01 codec**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_stdio_session.rs`

- [x] **Step 1: Add session tests**

Add tests for matching success, mismatched ids, EOF, and summary counters.

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_session
```

### Task 3: Implement Session Runtime

**Files:**
- Modify: `crates/ruse-mcp/src/client_stdio.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add transport trait and fixture transport**
- [x] **Step 2: Add session state and summary**
- [x] **Step 3: Add request/response matching**

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m43 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_session
```

### Task 5: Commit

- [x] **Step 1: Commit M43-02**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp client stdio session"
```
