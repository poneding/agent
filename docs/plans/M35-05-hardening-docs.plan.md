# M35-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M35 documentation, hardening checks, and final milestone status.

**Architecture:** Documentation and final verification slice.

**Tech Stack:** Markdown docs, Cargo workspace quality gates.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M35 specs and completed implementation slices**

### Task 2: Update Docs

- [x] **Step 1: Update README prompt export usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M35 focused tests**
- [x] **Step 2: Run `cargo xtask ci`**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-mcp mcp_server_prompt_export
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-mcp mcp_server_prompt_protocol
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-mcp mcp_server_manifest_prompts
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-cli mcp_server_prompt_profile
CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-cli mcp_server_profile_inspect_command
CARGO_TARGET_DIR=target/codex-m35 cargo xtask ci
```

Quality gate evidence:

```text
Quality gate ci-local status=passed
total=4 passed=4 failed=0 skipped=0
fmt-check status=passed exit=0
clippy status=passed exit=0
test status=passed exit=0
eval status=passed exit=0
```

### Task 4: Commit

```bash
git add README.md docs crates
git commit -m "feat: complete m35 mcp prompt export runtime"
```

- [x] **Step 1: Commit M35-05**
