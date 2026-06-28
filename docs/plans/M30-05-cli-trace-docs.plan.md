# M30-05 CLI Trace Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe CLI/status documentation for MCP server mode and complete M30.

**Architecture:** CLI renders `agent-mcp` summaries; runtime logic remains in domain crates.

**Tech Stack:** Rust 2024 edition, serde JSON, Markdown docs, Cargo quality gates.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M30-05 spec and completed M30 slice contracts**

### Task 2: Write Tests

- [ ] **Step 1: Add CLI status and JSON render tests**

### Task 3: Implement

- [ ] **Step 1: Add MCP server status command surface**
- [ ] **Step 2: Add trace-safe export summaries**
- [ ] **Step 3: Update README and milestone status**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask ci
```

```bash
git add README.md docs crates
git commit -m "feat: complete m30 mcp server export"
```
