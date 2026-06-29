# M45-01 HTTP Client Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure JSON-RPC HTTP client contracts and tests.

**Architecture:** Implement pure helpers in `agent-mcp` and export stable contracts from `lib.rs`; no live network dependency in this slice.

**Tech Stack:** Rust 2024 edition, serde JSON, async-trait only if needed, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M45-01-http-client-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_stdio.rs`
- Read: `crates/agent-mcp/src/server.rs`

- [ ] **Step 1: Review existing JSON-RPC codec patterns**

### Task 2: Write Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_contracts.rs`

- [ ] **Step 1: Add request rendering and response parsing tests**
- [ ] **Step 2: Add redaction regression tests**

### Task 3: Implement Contracts

**Files:**
- Create: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add HTTP request/response structs and helpers**
- [ ] **Step 2: Export public contracts**

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m45 cargo test -p agent-mcp mcp_client_http_contracts
CARGO_TARGET_DIR=target/codex-m45 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m45 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Commit M45-01**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http client contracts"
```
