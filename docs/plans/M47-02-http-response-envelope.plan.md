# M47-02 HTTP Response Envelope Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a typed HTTP response envelope carrying response mode plus safe session metadata.

**Architecture:** `agent-mcp::client_http` owns envelope contracts and updates fixture/reqwest transport return types.

**Tech Stack:** Rust 2024 edition, async-trait, reqwest, serde, Cargo fmt, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M47-02-http-response-envelope.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`
- Read: `crates/agent-mcp/tests/mcp_client_http_transport.rs`

- [x] **Step 1: Review response-mode and transport contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_client_http_response_envelope.rs`
- Modify: `crates/agent-mcp/tests/mcp_client_http_transport.rs`

- [x] **Step 1: Add envelope and transport metadata tests**

### Task 3: Implement Envelope Integration

**Files:**
- Modify: `crates/agent-mcp/src/client_http.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add response envelope contract**
- [x] **Step 2: Update fixture transport conversions**
- [x] **Step 3: Update reqwest transport header extraction**
- [x] **Step 4: Preserve response-mode frame parsing behavior**

### Task 4: Verify

- [x] **Step 1: Run focused tests and fmt**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_response_envelope
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_transport
CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M47-02**

Run:

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp http response envelopes"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_response_envelope` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_transport` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo clippy -p agent-mcp --all-targets -- -D warnings` passed.
