# M46-02 HTTP Response Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed JSON/SSE HTTP response-mode contracts.

**Architecture:** `ruse-mcp::client_http` maps response modes into existing JSON-RPC frame contracts.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M46-01 parser contracts and this spec**

### Task 2: Write Tests

- [x] **Step 1: Add `mcp_client_http_response_mode` tests**

### Task 3: Implement Response Mode

- [x] **Step 1: Add response-mode types and frame extraction helpers**
- [x] **Step 2: Preserve existing JSON response behavior**

### Task 4: Verify

- [x] **Step 1: Run focused tests and clippy**

### Task 5: Commit

- [x] **Step 1: Commit M46-02**

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_response_mode` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_sse_contracts` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_contracts` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m46 cargo clippy -p ruse-mcp --all-targets -- -D warnings` passed.
