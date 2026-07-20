# M36-03 Manifest Prompt Arguments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Load prompt argument declarations from export manifests.

**Architecture:** `ruse-mcp` manifest code owns parsing and profile construction.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M36-03 spec and manifest prompt profile code**

### Task 2: Write Failing Tests

- [x] **Step 1: Add manifest argument parse and serialization tests**
- [x] **Step 2: Add duplicate argument rejection tests**
- [x] **Step 3: Add inspection argument summary tests**

### Task 3: Implement Manifest Arguments

- [x] **Step 1: Add argument config structs**
- [x] **Step 2: Build prompt exports with arguments**
- [x] **Step 3: Include argument counts and names in inspection**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-mcp mcp_server_manifest_prompt_arguments
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-mcp mcp_server_manifest_prompt_arguments
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp manifest prompt arguments"
```

- [x] **Step 1: Commit M36-03**
