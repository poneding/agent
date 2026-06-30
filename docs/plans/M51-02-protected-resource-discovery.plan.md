# M51-02 Protected Resource Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Parse protected-resource and OAuth metadata JSON into typed contracts.

**Architecture:** `agent-mcp::client_http` owns parsing helpers without live network discovery.

**Tech Stack:** Rust 2024 edition, serde_json, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M51-02-protected-resource-discovery.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review discovery spec and parsing helpers**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_discovery.rs`

- [ ] **Step 1: Add discovery parser tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo test -p agent-mcp mcp_client_http_oauth_discovery
```

### Task 3: Implement Discovery Parsing

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add OAuth metadata JSON parser**
- [ ] **Step 2: Add protected resource metadata JSON parser**
- [ ] **Step 3: Add redacted parser diagnostics**
- [ ] **Step 4: Export parser helpers**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M51-02**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http oauth metadata discovery"
```
