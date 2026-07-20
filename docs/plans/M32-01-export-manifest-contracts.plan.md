# M32-01 Export Manifest Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP server export manifest contracts in `ruse-mcp`.

**Architecture:** `ruse-mcp::server_manifest` owns serde and validation; no CLI or file loading in this slice.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M32-01 spec and existing profile validation code**

### Task 2: Write Tests

- [x] **Step 1: Add manifest serde and validation tests**

### Task 3: Implement

- [x] **Step 1: Add manifest contracts**
- [x] **Step 2: Add parse and selection helpers**
- [x] **Step 3: Export contracts from `ruse_mcp`**

### Task 4: Verify and Commit

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m32 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m32 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-mcp mcp_server_manifest
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-mcp
```

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-mcp --all-targets -- -D warnings
cargo test -p ruse-mcp mcp_server_manifest
```

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp export manifest contracts"
```
