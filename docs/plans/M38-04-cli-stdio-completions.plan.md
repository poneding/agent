# M38-04 CLI Stdio Completions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire manifest-backed MCP completions into CLI stdio `completion/complete`.

**Architecture:** `agent-cli` loads manifest completion declarations into profile runtime; `agent-mcp` handles JSON-RPC dispatch and completion lookup.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M38-04 spec, server stdio dispatch, and manifest loader**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add manifest prompt completion tests**
- [ ] **Step 2: Add manifest resource completion tests**
- [ ] **Step 3: Add CLI stdio `completion/complete` tests**

### Task 3: Implement Manifest and Stdio Wiring

- [ ] **Step 1: Add manifest completion fields and validation**
- [ ] **Step 2: Load completion sources into profile runtime**
- [ ] **Step 3: Add `completion/complete` method dispatch**
- [ ] **Step 4: Keep stdio stdout protocol-only**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m38 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m38 cargo clippy -p agent-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m38 cargo test -p agent-mcp mcp_server_manifest_completions
CARGO_TARGET_DIR=target/codex-m38 cargo test -p agent-cli mcp_server_completions
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: wire mcp stdio completions"
```

- [ ] **Step 1: Commit M38-04**
