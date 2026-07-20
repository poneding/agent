# M32-04 CLI Profile Stdio Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI profile flags for manifest-backed MCP stdio startup.

**Architecture:** CLI owns file reading and args; `ruse-mcp` owns manifest/profile semantics.

**Tech Stack:** Rust 2024 edition, stdio, subprocess tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M32-04 spec and current stdio command runner**

### Task 2: Write Tests

- [x] **Step 1: Add parse and subprocess tests**

### Task 3: Implement

- [x] **Step 1: Add stdio profile flags**
- [x] **Step 2: Load manifest before stdio loop**
- [x] **Step 3: Preserve protocol-only stdout**

### Task 4: Verify and Commit

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m32 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m32 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m32 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-cli mcp_server_profile_command
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-cli
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-mcp
```

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-cli --all-targets -- -D warnings
cargo test -p ruse-cli mcp_server_profile_command
```

```bash
git add crates/ruse-cli docs
git commit -m "feat: add mcp stdio profile command"
```
