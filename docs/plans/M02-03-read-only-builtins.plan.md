# M02-03 Read-Only Built-ins Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add sandboxed read-only workspace tools for file reads and text search.

**Architecture:** `agent-sandbox` owns workspace path confinement. `agent-tool` owns built-in read-only tools and calls the sandbox before reading local files.

**Tech Stack:** Rust 2024 edition, `agent-tool`, `agent-sandbox`, `serde_json`, `tempfile` for tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Sandbox Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/agent-sandbox/Cargo.toml`
- Create: `crates/agent-sandbox/src/lib.rs`
- Create: `crates/agent-sandbox/src/path.rs`

- [ ] **Step 1: Add workspace member and manifest**

Dependencies:

```toml
[dependencies]
thiserror = "2"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Write path tests**

Create tests for normal relative paths, `..` traversal, absolute paths, and symlink escape.

### Task 2: Implement Workspace Sandbox

**Files:**
- Create: `crates/agent-sandbox/tests/path.rs`
- Modify: `crates/agent-sandbox/src/path.rs`

- [ ] **Step 1: Verify RED**

```bash
cargo test -p agent-sandbox path
```

Expected: tests fail before path confinement exists.

- [ ] **Step 2: Implement `WorkspaceSandbox`**

Implement `resolve_read_path(&self, relative_path: &str)` and reject paths outside the root.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-sandbox path
```

Expected: sandbox tests pass.

### Task 3: Add Built-in Tools

**Files:**
- Modify: `crates/agent-tool/Cargo.toml`
- Create: `crates/agent-tool/src/builtins/mod.rs`
- Create: `crates/agent-tool/src/builtins/read_file.rs`
- Create: `crates/agent-tool/src/builtins/search_text.rs`
- Create: `crates/agent-tool/tests/read_only_builtins.rs`

- [ ] **Step 1: Write failing built-in tests**

Tests must prove read file returns content, search returns matching lines, traversal is rejected, and tools can be registered.

- [ ] **Step 2: Implement tools**

Implement `ReadFileTool` and `SearchTextTool` as `Tool` implementations using `WorkspaceSandbox`.

- [ ] **Step 3: Verify focused tests**

```bash
cargo test -p agent-tool read_only_builtins
```

Expected: built-in tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M02-03-read-only-builtins.plan.md`

- [ ] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 2: Advance active focus**

Update active focus from `M02-03-read-only-builtins` to `M02-04-audit-trail`.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m02-03 read only builtins"
```
