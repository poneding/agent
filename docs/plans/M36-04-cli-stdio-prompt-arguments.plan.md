# M36-04 CLI Stdio Prompt Arguments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify prompt arguments through `ruse-cli mcp server stdio`.

**Architecture:** `ruse-cli` stays an IO/bootstrap layer; `ruse-mcp` owns rendering.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M36-04 spec and CLI stdio tests**

### Task 2: Write Failing Tests

- [x] **Step 1: Add prompts/list descriptor e2e test**
- [x] **Step 2: Add prompts/get arguments e2e test**
- [x] **Step 3: Add invalid argument e2e test**

### Task 3: Implement CLI Wiring

- [x] **Step 1: Parse `arguments` in stdio `prompts/get` params**
- [x] **Step 2: Preserve protocol-only stdout**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-cli mcp_server_prompt_arguments
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-cli mcp_server_prompt_arguments
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: wire mcp stdio prompt arguments"
```

- [x] **Step 1: Commit M36-04**
