# M35-03 Manifest Prompt Profile Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add manifest-backed prompt export declarations.

**Architecture:** `agent-mcp` manifest code owns parsing and profile construction.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M35-03 spec and current export manifest code**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add manifest prompt parse and serialization tests**
- [ ] **Step 2: Add disabled prompt hiding tests**
- [ ] **Step 3: Add inspection prompt summary tests**

### Task 3: Implement Manifest Prompts

- [ ] **Step 1: Add prompt config structs**
- [ ] **Step 2: Add prompt profile builder path**
- [ ] **Step 3: Include prompt inspections**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-mcp mcp_server_manifest_prompts
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp manifest prompt exports"
```

- [ ] **Step 1: Commit M35-03**
