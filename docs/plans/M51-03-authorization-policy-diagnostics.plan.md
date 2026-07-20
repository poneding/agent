# M51-03 Authorization Policy Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add redacted authorization readiness diagnostics for MCP HTTP clients.

**Architecture:** Pure DTO/report helpers in `ruse-mcp::client_http`.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M51-03-authorization-policy-diagnostics.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`

- [x] **Step 1: Review authorization diagnostic scope**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_authorization_diagnostics.rs`

- [x] **Step 1: Add diagnostic tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_authorization_diagnostics
```

### Task 3: Implement Diagnostics

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add diagnostic severity and DTOs**
- [x] **Step 2: Add readiness report builder**
- [x] **Step 3: Add header conflict summaries**
- [x] **Step 4: Export contracts**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M51-03**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp http authorization diagnostics"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_authorization_diagnostics`
- `CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p ruse-mcp --all-targets -- -D warnings`
