# M47-01 HTTP Session Metadata Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure contracts for safe MCP streamable HTTP session metadata.

**Architecture:** `agent-mcp::client_http` owns metadata parsing and exports contracts through `agent-mcp`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M47-01-http-session-metadata-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Review the spec and existing HTTP contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_session_metadata.rs`

- [ ] **Step 1: Add focused metadata parser tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_session_metadata
```

Expected: tests fail before metadata contracts exist.

### Task 3: Implement Metadata Contracts

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add `McpClientHttpSessionMetadata`**
- [ ] **Step 2: Add metadata parsing and validation helpers**
- [ ] **Step 3: Export public contracts**
- [ ] **Step 4: Run focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_session_metadata
```

Expected: focused tests pass.

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m47 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

Expected: checks pass.

### Task 5: Commit

- [ ] **Step 1: Update status and commit M47-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http session metadata contracts"
```
