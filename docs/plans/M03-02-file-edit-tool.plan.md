# M03-02 File Edit Tool Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a workspace-confined file edit tool with preview and apply modes.

**Architecture:** `agent-tool` implements `FileEditTool` in built-ins and uses `agent-sandbox` for path confinement. The tool emits deterministic JSON output so later CLI and audit layers can render diffs.

**Tech Stack:** Rust 2024 edition, `agent-tool`, `agent-sandbox`, `serde_json`, `tempfile`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write File Edit Tool Tests

**Files:**
- Create: `crates/agent-tool/tests/file_edit_tool.rs`

- [ ] **Step 1: Add failing tests**

Tests must prove preview does not mutate, apply mutates, expected mismatch rejects, traversal rejects, and registry registration works.

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-tool file_edit_tool
```

Expected: tests fail because `FileEditTool` does not exist.

### Task 2: Implement File Edit Tool

**Files:**
- Create: `crates/agent-tool/src/builtins/edit_file.rs`
- Modify: `crates/agent-tool/src/builtins/mod.rs`

- [ ] **Step 1: Implement preview mode**

Return JSON `{ "path": ..., "applied": false, "diff": ... }` without writing the file.

- [ ] **Step 2: Implement apply mode**

Write replacement content only after sandbox resolution and optional expected-content match.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-tool file_edit_tool
```

Expected: focused tests pass.

### Task 3: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M03-02-file-edit-tool.plan.md`

- [ ] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 2: Advance active focus**

Update active focus from `M03-02-file-edit-tool` to `M03-03-shell-tool`.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m03-02 file edit tool"
```
