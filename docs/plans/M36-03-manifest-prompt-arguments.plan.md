# M36-03 Manifest Prompt Arguments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Load prompt argument declarations from export manifests.

**Architecture:** `agent-mcp` manifest code owns parsing and profile construction.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M36-03 spec and manifest prompt profile code**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add manifest argument parse and serialization tests**
- [ ] **Step 2: Add duplicate argument rejection tests**
- [ ] **Step 3: Add inspection argument summary tests**

### Task 3: Implement Manifest Arguments

- [ ] **Step 1: Add argument config structs**
- [ ] **Step 2: Build prompt exports with arguments**
- [ ] **Step 3: Include argument counts and names in inspection**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m36 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m36 cargo test -p agent-mcp mcp_server_manifest_prompt_arguments
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp manifest prompt arguments"
```

- [ ] **Step 1: Commit M36-03**
