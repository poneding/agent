# M51-02 Protected Resource Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Parse protected-resource and OAuth metadata JSON into typed contracts.

**Architecture:** `ruse-mcp::client_http` owns parsing helpers without live network discovery.

**Tech Stack:** Rust 2024 edition, serde_json, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M51-02-protected-resource-discovery.spec.md`
- Read: `crates/ruse-mcp/src/client_http.rs`

- [x] **Step 1: Review discovery spec and parsing helpers**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client_http_oauth_discovery.rs`

- [x] **Step 1: Add discovery parser tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_discovery
```

### Task 3: Implement Discovery Parsing

**Files:**
- Modify: `crates/ruse-mcp/src/client_http.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add OAuth metadata JSON parser**
- [x] **Step 2: Add protected resource metadata JSON parser**
- [x] **Step 3: Add redacted parser diagnostics**
- [x] **Step 4: Export parser helpers**
- [x] **Step 5: Run focused tests**

### Task 4: Verify

- [x] **Step 1: Run focused quality checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p ruse-mcp --all-targets -- -D warnings
```

### Task 5: Commit

- [x] **Step 1: Update status and commit M51-02**

Run:

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp http oauth metadata discovery"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_discovery`
- `CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m51 cargo clippy -p ruse-mcp --all-targets -- -D warnings`
