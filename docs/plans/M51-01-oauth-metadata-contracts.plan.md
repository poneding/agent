# M51-01 OAuth Metadata Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure OAuth metadata contracts for MCP HTTP authorization.

**Architecture:** `agent-mcp::client_http` owns metadata contracts and exports them through `agent-mcp`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M51-01-oauth-metadata-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Review metadata spec and existing HTTP contract patterns**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_metadata.rs`

- [ ] **Step 1: Add focused OAuth metadata tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo test -p agent-mcp mcp_client_http_oauth_metadata
```

Expected: tests fail before metadata contracts exist.

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add OAuth metadata DTO**
- [ ] **Step 2: Add protected resource metadata DTO**
- [ ] **Step 3: Add validation helpers and redacted errors**
- [ ] **Step 4: Export public contracts**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M51-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http oauth metadata contracts"
```
