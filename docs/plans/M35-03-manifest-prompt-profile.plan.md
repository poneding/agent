# M35-03 Manifest Prompt Profile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add manifest-backed prompt export declarations.

**Architecture:** `ruse-mcp` manifest code owns parsing and profile construction.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M35-03 spec and current export manifest code**

### Task 2: Write Failing Tests

- [x] **Step 1: Add manifest prompt parse and serialization tests**
- [x] **Step 2: Add disabled prompt hiding tests**
- [x] **Step 3: Add inspection prompt summary tests**

### Task 3: Implement Manifest Prompts

- [x] **Step 1: Add prompt config structs**
- [x] **Step 2: Add prompt profile builder path**
- [x] **Step 3: Include prompt inspections**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p ruse-mcp mcp_server_manifest_prompts
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p ruse-mcp mcp_server_manifest_prompts
CARGO_TARGET_DIR=target/codex-m35 cargo test -p ruse-mcp
CARGO_TARGET_DIR=target/codex-m35 cargo check -p ruse-cli
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p ruse-cli --all-targets -- -D warnings
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp manifest prompt exports"
```

- [x] **Step 1: Commit M35-03**
