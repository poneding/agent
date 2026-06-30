# M47-03 Session Header Propagation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist safe HTTP session metadata after initialize and add session headers to later requests.

**Architecture:** `McpHttpClientState` owns session metadata; request construction remains in `agent-mcp::client_http`.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Cargo fmt, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M47-03-session-header-propagation.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_mcp_client.rs`

- [ ] **Step 1: Review client session state**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_session_headers.rs`

- [ ] **Step 1: Add session header propagation tests**

### Task 3: Implement Session State

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Store metadata after successful initialize**
- [ ] **Step 2: Add session-aware request construction**
- [ ] **Step 3: Reject conflicting headers safely**

### Task 4: Verify

- [ ] **Step 1: Run focused tests and fmt**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_session_headers
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_mcp_client
CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M47-03**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: propagate mcp http session headers"
```
