# M34-01 Profile Inspection Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP export profile inspection contracts in `ruse-mcp`.

**Architecture:** `ruse-mcp` owns inspection semantics; no CLI logic in this slice.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M34-01 spec and current manifest helpers**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_server_profile_inspection.rs`

- [x] **Step 1: Add resource-only inspection test**
- [x] **Step 2: Add missing workspace diagnostic test**
- [x] **Step 3: Add disabled entries and serde shape tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-mcp mcp_server_profile_inspection
```

Expected: tests fail before implementation.

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/ruse-mcp/src/server_manifest.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add inspection data types**
- [x] **Step 2: Add inspection builder**
- [x] **Step 3: Keep diagnostics redacted and bounded**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-mcp mcp_server_profile_inspection
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-mcp
```

- [x] **Step 1: Run focused and crate tests**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-mcp mcp_server_profile_inspection
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-mcp
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp profile inspection contracts"
```

- [x] **Step 1: Commit M34-01**
