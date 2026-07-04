# M02-01 Tool Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the `agent-tool` crate with typed tool definitions, execution contracts, and duplicate-safe registry lookup.

**Architecture:** `agent-tool` owns provider-neutral tool contracts and depends only on `agent-types`, `serde`, `serde_json`, and `thiserror`. `agent-core` will consume these contracts in later slices.

**Tech Stack:** Rust 2024 edition, `agent-types`, `serde`, `serde_json`, `thiserror`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Tool Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/agent-tool/Cargo.toml`
- Create: `crates/agent-tool/src/lib.rs`

- [x] **Step 1: Add workspace member**

Add `crates/agent-tool` to the root workspace members.

- [x] **Step 2: Add crate manifest**

Use dependencies:

```toml
[dependencies]
agent-types = { path = "../agent-types" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

- [x] **Step 3: Export modules**

`src/lib.rs` should export `definition`, `error`, `registry`, and `tool`.

### Task 2: Write Failing Registry Tests

**Files:**
- Create: `crates/agent-tool/tests/registry.rs`

- [x] **Step 1: Add contract tests**

Tests must prove:

- valid tool names are accepted
- invalid names are rejected
- a registered tool can be looked up and executed
- duplicate names are rejected
- unknown lookup returns `None`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-tool registry
```

Expected: tests fail because contracts are not implemented.

### Task 3: Implement Tool Contracts

**Files:**
- Create: `crates/agent-tool/src/definition.rs`
- Create: `crates/agent-tool/src/error.rs`
- Create: `crates/agent-tool/src/tool.rs`
- Create: `crates/agent-tool/src/registry.rs`

- [x] **Step 1: Implement public types**

Implement `ToolName`, `ToolDefinition`, `ToolInput`, `ToolOutput`, `ToolError`, `ToolRegistryError`, and `Tool`.

- [x] **Step 2: Implement registry**

Use a `BTreeMap<ToolName, Arc<dyn Tool>>` so listing and lookup are deterministic.

- [x] **Step 3: Verify GREEN**

Run:

```bash
cargo test -p agent-tool registry
```

Expected: focused tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M02-01-tool-registry.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M02-01-tool-registry` to `M02-02-policy-engine`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m02-01 tool registry"
```
