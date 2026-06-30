# M53-02 Authorization Request Planning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic OAuth authorization request planning.

**Architecture:** Pure `agent-mcp::client_http` helper over M51 metadata and M53 PKCE/state contracts.

**Tech Stack:** Rust 2024 edition, serde, URL encoding helper as needed, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M53-02-authorization-request-planning.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review authorization request scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_authorization_request.rs`

- [ ] **Step 1: Add authorization request planning tests**

### Task 3: Implement Planning

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add input and plan DTOs**
- [ ] **Step 2: Add deterministic URL construction**
- [ ] **Step 3: Add validation and redacted errors**
- [ ] **Step 4: Export contracts**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

### Task 5: Commit

- [ ] **Step 1: Update status and commit M53-02**
