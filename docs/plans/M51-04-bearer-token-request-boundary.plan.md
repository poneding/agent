# M51-04 Bearer Token Request Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit bearer-token application to MCP HTTP request construction.

**Architecture:** `agent-mcp::client_http` applies caller-provided bearer authorization at request construction only.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M51-04-bearer-token-request-boundary.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review request builder patterns**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_bearer_authorization.rs`

- [ ] **Step 1: Add bearer authorization tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo test -p agent-mcp mcp_client_http_bearer_authorization
```

### Task 3: Implement Request Boundary

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add bearer authorization DTO**
- [ ] **Step 2: Add POST request builder with authorization**
- [ ] **Step 3: Add GET stream request builder with authorization**
- [ ] **Step 4: Add conflict and redaction handling**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M51-04**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http bearer authorization boundary"
```
