# M32-04 CLI Profile Stdio Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI profile flags for manifest-backed MCP stdio startup.

**Architecture:** CLI owns file reading and args; `agent-mcp` owns manifest/profile semantics.

**Tech Stack:** Rust 2024 edition, stdio, subprocess tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M32-04 spec and current stdio command runner**

### Task 2: Write Tests

- [ ] **Step 1: Add parse and subprocess tests**

### Task 3: Implement

- [ ] **Step 1: Add stdio profile flags**
- [ ] **Step 2: Load manifest before stdio loop**
- [ ] **Step 3: Preserve protocol-only stdout**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-cli --all-targets -- -D warnings
cargo test -p agent-cli mcp_server_profile_command
```

```bash
git add crates/agent-cli docs
git commit -m "feat: add mcp stdio profile command"
```
