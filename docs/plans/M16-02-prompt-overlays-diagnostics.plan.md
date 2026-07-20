# M16-02 Prompt Overlays and Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic prompt overlay application and assembly diagnostics.

**Architecture:** Keep all assembly logic pure inside `ruse-context`. `PromptRuntime` consumes M16-01 contracts and returns `PromptBundle` plus `PromptAssemblyReport`; no storage, CLI, UI bridge, or model provider code changes belong in this slice.

**Tech Stack:** Rust 2024 edition, serde contracts, existing context budget planner, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M16-02-prompt-overlays-diagnostics.spec.md`
- Read: `docs/specs/M16-00-prompt-runtime-technical-design.spec.md`
- Read: `crates/ruse-context/src/prompt.rs`
- Read: `crates/ruse-context/src/budget.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,240p' docs/specs/M16-02-prompt-overlays-diagnostics.spec.md
```

Expected: the spec defines pure overlay and diagnostic behavior only.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-context/tests/prompt_overlays.rs`

- [x] **Step 1: Add overlay and diagnostics tests**

Add tests named:

- `prompt_runtime_requires_profile_layers`
- `prompt_runtime_appends_overlay_layers_by_priority`
- `prompt_runtime_replaces_optional_layer_by_id`
- `prompt_runtime_rejects_required_hidden_layer_disable`
- `prompt_runtime_records_duplicate_optional_layer_diagnostic`
- `prompt_runtime_omits_optional_layers_when_budget_is_exceeded`
- `prompt_diagnostics_do_not_include_layer_content`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-context --test prompt_overlays
```

Expected: tests fail because prompt overlays, prompt runtime, reports, and diagnostics are not implemented.

### Task 3: Implement Prompt Runtime

**Files:**
- Modify: `crates/ruse-context/src/prompt.rs`
- Modify: `crates/ruse-context/src/lib.rs`

- [x] **Step 1: Add overlay and diagnostic contracts**

Implement:

- `PromptOverlay`
- `PromptOverlayAction`
- `PromptAssemblyRequest`
- `PromptAssemblyReport`
- `PromptDiagnostic`
- `PromptDiagnosticSeverity`

- [x] **Step 2: Add pure runtime assembly**

Implement `PromptRuntime::assemble(request)`.

Rules:

- validate profile and layers
- apply overlays by descending overlay priority with stable tie behavior
- enforce required layer ids
- sort final layers through existing bundle assembly
- return bundle plus report

- [x] **Step 3: Integrate budget behavior**

Use profile default budget to include required layers first and omit optional layers that exceed available budget.

- [x] **Step 4: Keep diagnostics content-safe**

Ensure diagnostic messages reference ids, source labels, and codes only.

### Task 4: Verify And Update Status

**Files:**
- Modify: `docs/plans/M16-02-prompt-overlays-diagnostics.plan.md`
- Modify: `docs/plans/M16-prompt-runtime-dynamic-management.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-context --test prompt_overlays
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: formatting, clippy, and workspace tests pass.

- [x] **Step 3: Mark M16-02 complete**

Update M16 planning status so active work advances to M16-03.

- [ ] **Step 4: Commit**

Run:

```bash
git add crates/ruse-context docs
git commit -m "feat: add m16 prompt overlays diagnostics"
```

Expected: commit succeeds if this slice is landed independently.
