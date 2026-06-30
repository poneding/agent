# M53-01 PKCE State Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add PKCE verifier/challenge and OAuth state contracts.

**Architecture:** Pure `agent-mcp::client_http` contracts; caller owns randomness and durable flow storage.

**Tech Stack:** Rust 2024 edition, serde, SHA-256/base64url helpers as needed, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M53-01-pkce-state-contracts.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review PKCE/state scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_pkce_state.rs`

- [ ] **Step 1: Add PKCE/state tests**

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`
- Modify: `crates/agent-mcp/Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Add PKCE verifier and state contracts**
- [ ] **Step 2: Add S256 challenge generation**
- [ ] **Step 3: Add redacted debug behavior**
- [ ] **Step 4: Export contracts**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

### Task 5: Commit

- [ ] **Step 1: Update status and commit M53-01**
