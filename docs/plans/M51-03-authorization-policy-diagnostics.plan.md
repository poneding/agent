# M51-03 Authorization Policy Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redacted authorization readiness diagnostics for MCP HTTP clients.

**Architecture:** Pure DTO/report helpers in `agent-mcp::client_http`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M51-03-authorization-policy-diagnostics.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review authorization diagnostic scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_authorization_diagnostics.rs`

- [ ] **Step 1: Add diagnostic tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo test -p agent-mcp mcp_client_http_authorization_diagnostics
```

### Task 3: Implement Diagnostics

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add diagnostic severity and DTOs**
- [ ] **Step 2: Add readiness report builder**
- [ ] **Step 3: Add header conflict summaries**
- [ ] **Step 4: Export contracts**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p agent-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [ ] **Step 1: Update status and commit M51-03**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http authorization diagnostics"
```
