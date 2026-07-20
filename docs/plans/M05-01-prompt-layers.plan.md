# M05-01 Prompt Layers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the `ruse-context` crate and deterministic prompt layer assembly contracts.

**Architecture:** `ruse-context` depends only on `ruse-types`, `serde`, `serde_json`, and `thiserror`. It owns prompt assembly contracts and exposes pure functions that `ruse-core` can consume later.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold `ruse-context`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-context/Cargo.toml`
- Create: `crates/ruse-context/src/lib.rs`
- Create: `crates/ruse-context/src/error.rs`
- Create: `crates/ruse-context/src/prompt.rs`

- [x] **Step 1: Add failing crate smoke test**

Create `crates/ruse-context/tests/prompt_layers.rs` with:

```rust
use ruse_context::PromptLayerKind;

#[test]
fn prompt_layers_expose_public_kind_contract() {
    assert_eq!(serde_json::to_string(&PromptLayerKind::System).unwrap(), "\"system\"");
}
```

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-context prompt_layers_expose_public_kind_contract
```

Expected: fails because `ruse-context` is not a workspace crate yet.

- [x] **Step 3: Add crate scaffold**

Add `crates/ruse-context` to workspace members. Create `Cargo.toml` with dependencies on `ruse-types`, `serde`, `serde_json`, and `thiserror`. Create `lib.rs` that re-exports `ContextError`, `PromptBundle`, `PromptLayer`, `PromptLayerKind`, `PromptMessage`, and `assemble_prompt_bundle`.

- [x] **Step 4: Verify crate smoke GREEN**

```bash
cargo test -p ruse-context prompt_layers_expose_public_kind_contract
```

Expected: the smoke test passes.

### Task 2: Add Prompt Layer Contract Tests

**Files:**
- Modify: `crates/ruse-context/tests/prompt_layers.rs`

- [x] **Step 1: Add failing behavior tests**

Add tests named:

- `prompt_layers_sort_by_priority_and_preserve_ties`
- `prompt_layers_reject_empty_id`
- `prompt_layers_reject_empty_content`
- `prompt_layers_serialize_stable_shape`

Use `assemble_prompt_bundle` with layers for `system`, `project`, and `user`; assert ordered message contents and total `estimated_tokens`.

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-context prompt_layers
```

Expected: tests fail because prompt assembly is not implemented.

### Task 3: Implement Prompt Assembly

**Files:**
- Modify: `crates/ruse-context/src/error.rs`
- Modify: `crates/ruse-context/src/prompt.rs`
- Modify: `crates/ruse-context/src/lib.rs`

- [x] **Step 1: Implement public contracts**

Define `PromptLayerKind`, `PromptLayer`, `PromptMessage`, `PromptBundle`, and `ContextError`. Derive `Clone`, `Debug`, `Eq`, `PartialEq`, `Serialize`, and `Deserialize` where durable or test-facing.

- [x] **Step 2: Implement deterministic ordering**

Implement `assemble_prompt_bundle` by validating each layer, sorting by descending `priority`, preserving tie order with an enumerated index, mapping kinds to roles, and summing `token_estimate`.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p ruse-context prompt_layers
```

Expected: prompt layer tests pass.

### Task 4: Workspace Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M05-01-prompt-layers.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M05-01-prompt-layers` to `M05-02-budget-planner`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m05 prompt layers"
```
