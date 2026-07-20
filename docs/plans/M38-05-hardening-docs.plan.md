# M38-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M38 documentation, hardening checks, and final milestone status.

**Architecture:** Documentation and final verification slice.

**Tech Stack:** Markdown docs, Cargo workspace quality gates.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M38 specs and completed implementation slices**

### Task 2: Update Docs

- [x] **Step 1: Update README completion usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M38 focused tests**
- [x] **Step 2: Run `cargo xtask ci`**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m38 cargo test -p ruse-mcp mcp_server_completion_contracts
CARGO_TARGET_DIR=target/codex-m38 cargo test -p ruse-mcp mcp_server_prompt_completions
CARGO_TARGET_DIR=target/codex-m38 cargo test -p ruse-mcp mcp_server_resource_template_completions
CARGO_TARGET_DIR=target/codex-m38 cargo test -p ruse-mcp mcp_server_manifest_completions
CARGO_TARGET_DIR=target/codex-m38 cargo test -p ruse-cli mcp_server_completions
CARGO_TARGET_DIR=target/codex-m38 cargo xtask ci
```

### Task 4: Commit

```bash
git add README.md docs crates
git commit -m "feat: complete m38 mcp completion runtime"
```

- [x] **Step 1: Commit M38-05**
