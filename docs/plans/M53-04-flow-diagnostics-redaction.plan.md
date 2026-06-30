# M53-04 Flow Diagnostics Redaction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add serializable, redacted OAuth flow diagnostics.

**Architecture:** `agent-mcp::client_http` diagnostic contracts reused by M53 flow helpers.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M53-04-flow-diagnostics-redaction.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [ ] **Step 1: Review diagnostic scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_oauth_flow_diagnostics.rs`

- [ ] **Step 1: Add flow diagnostic tests**

### Task 3: Implement Diagnostics

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [ ] **Step 1: Add diagnostic severity/code contracts**
- [ ] **Step 2: Add redacted builder helpers**
- [ ] **Step 3: Wire diagnostics into flow helpers**
- [ ] **Step 4: Export contracts**
- [ ] **Step 5: Run focused tests**

### Task 4: Verify

- [ ] **Step 1: Run focused quality checks**

### Task 5: Commit

- [ ] **Step 1: Update status and commit M53-04**
