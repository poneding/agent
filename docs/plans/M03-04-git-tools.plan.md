# M03-04 Git Tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add read-only Git status and diff tools.

**Architecture:** `agent-tool` implements `GitStatusTool` and `GitDiffTool`, using `agent-sandbox` for cwd validation and `git` subprocesses for fixture-backed behavior.

**Tech Stack:** Rust 2024 edition, `agent-tool`, `agent-sandbox`, `serde_json`, `tempfile`, local `git`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Git Tool Tests

**Files:**
- Create: `crates/agent-tool/tests/git_tools.rs`

- [x] **Step 1: Add failing tests**

Tests must prove clean status, modified status, diff output, non-Git error, traversal rejection, and registry registration.

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-tool git_tools
```

Expected: tests fail because Git tools do not exist.

### Task 2: Implement Git Tools

**Files:**
- Create: `crates/agent-tool/src/builtins/git_status.rs`
- Create: `crates/agent-tool/src/builtins/git_diff.rs`
- Modify: `crates/agent-tool/src/builtins/mod.rs`

- [x] **Step 1: Implement status tool**

Run `git status --porcelain` inside validated cwd and return JSON file entries.

- [x] **Step 2: Implement diff tool**

Run `git diff -- <path>` inside validated cwd and return JSON `{ "diff": "..." }`.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p agent-tool git_tools
```

Expected: focused tests pass.

### Task 3: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M03-04-git-tools.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M03-04-git-tools` to `M03-05-local-coding-e2e`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m03-04 git tools"
```
