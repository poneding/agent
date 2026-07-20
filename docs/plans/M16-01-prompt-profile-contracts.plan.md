# M16-01 Prompt Profile Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add versioned prompt profile and prompt layer metadata contracts in `ruse-context`.

**Architecture:** Keep M16-01 entirely inside `ruse-context`. Extend existing prompt contracts without introducing runtime persistence, CLI commands, UI bridge payloads, or model-provider dependencies.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `ContextError`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M16-01-prompt-profile-contracts.spec.md`
- Read: `docs/specs/M16-00-prompt-runtime-technical-design.spec.md`
- Read: `crates/ruse-context/src/prompt.rs`
- Read: `crates/ruse-context/tests/prompt_layers.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M16-01-prompt-profile-contracts.spec.md
```

Expected: the spec defines foundational prompt profile and layer metadata contracts only.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-context/tests/prompt_profiles.rs`
- Modify: `crates/ruse-context/tests/prompt_layers.rs`

- [x] **Step 1: Add prompt profile tests**

Add tests named:

- `prompt_profile_accepts_valid_contract`
- `prompt_profile_rejects_empty_id`
- `prompt_profile_rejects_zero_version`
- `prompt_profile_rejects_duplicate_required_layers`
- `prompt_profile_serializes_stable_shape`

- [x] **Step 2: Extend prompt layer tests**

Add or update tests named:

- `prompt_layers_serialize_extended_metadata_shape`
- `prompt_layer_source_uses_stable_snake_case_shape`
- `prompt_layers_reject_empty_metadata_fields`

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-context --test prompt_profiles
cargo test -p ruse-context --test prompt_layers
```

Expected: tests fail because profile ids, profile contracts, source metadata, visibility, sensitivity, policy, and version fields are not implemented.

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/ruse-context/src/prompt.rs`
- Modify: `crates/ruse-context/src/error.rs`
- Modify: `crates/ruse-context/src/lib.rs`

- [x] **Step 1: Add ids and profile contracts**

Implement:

- `PromptProfileId`
- `PromptLayerId`
- `PromptBudget`
- `PromptProfile`

Validation rules:

- ids trim to non-empty strings
- display names trim to non-empty strings
- profile version must be greater than zero
- required layer ids must be unique
- budget must keep reserved response tokens lower than max tokens

- [x] **Step 2: Add layer metadata contracts**

Implement:

- `PromptLayerSource`
- `PromptLayerVisibility`
- `PromptLayerSensitivity`
- `PromptLayerPolicy`

Extend `PromptLayer` with:

- `source`
- `visibility`
- `sensitivity`
- `policy`
- `version`

- [x] **Step 3: Preserve bundle assembly**

Update `assemble_prompt_bundle` and existing tests so prompt layers still sort by descending priority and stable insertion order for ties.

- [x] **Step 4: Export contracts**

Update `crates/ruse-context/src/lib.rs` so downstream crates can use the new contracts.

### Task 4: Verify And Update Status

**Files:**
- Modify: `docs/plans/M16-01-prompt-profile-contracts.plan.md`
- Modify: `docs/plans/M16-prompt-runtime-dynamic-management.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-context --test prompt_profiles
cargo test -p ruse-context --test prompt_layers
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: formatting, clippy, and workspace tests pass.

- [x] **Step 3: Mark M16-01 complete**

Update M16 planning status so active work advances to M16-02.

- [ ] **Step 4: Commit**

Run:

```bash
git add crates/ruse-context docs
git commit -m "feat: add m16 prompt profile contracts"
```

Expected: commit succeeds if this slice is landed independently.
