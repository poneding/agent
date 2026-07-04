# M39-01 Client Root Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP client root contracts and capability metadata.

**Architecture:** `agent-mcp` owns root protocol contracts in a client-roots module exported from `lib.rs`.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M39-01 spec and existing MCP client contracts**

### Task 2: Write Failing Tests

- [x] **Step 1: Add root and root list serialization tests**
- [x] **Step 2: Add client capability serialization tests**
- [x] **Step 3: Add root-list notification serialization tests**
- [x] **Step 4: Add invalid and secret-like metadata tests**

### Task 3: Implement Contracts

- [x] **Step 1: Add root contracts module**
- [x] **Step 2: Add deterministic constructors and validators**
- [x] **Step 3: Export contracts from `agent-mcp`**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m39 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m39 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m39 cargo test -p agent-mcp mcp_client_root_contracts
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m39 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m39 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m39 cargo test -p agent-mcp mcp_client_root_contracts
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp client root contracts"
```

- [x] **Step 1: Commit M39-01**
