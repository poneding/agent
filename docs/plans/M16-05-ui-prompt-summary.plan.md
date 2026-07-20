# M16-05 UI Prompt Summary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add UI-safe prompt summary contracts shared by CLI JSON, future TUI, and Desktop clients.

**Architecture:** `ruse-ui-bridge` owns UI view models and normalization. It consumes prompt snapshot metadata from `ruse-types` and mirrors only safe prompt summary fields.

**Tech Stack:** Rust 2024 edition, serde UI bridge contracts, runtime event normalization, JSON frame snapshots, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M16-05-ui-prompt-summary.spec.md`
- Read: `crates/ruse-ui-bridge/src/event.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-ui-bridge/tests/bridge_snapshots.rs`
- Read: `crates/ruse-types/src/event.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M16-05-ui-prompt-summary.spec.md
```

Expected: the spec defines UI-safe prompt summary contracts and redaction requirements.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-ui-bridge/tests/prompt_summary.rs`
- Modify: `crates/ruse-ui-bridge/tests/bridge_snapshots.rs`

- [x] **Step 1: Add prompt summary tests**

Add tests named:

- `ui_prompt_summary_serializes_stable_shape`
- `ui_prompt_summary_hides_hidden_layer_preview`
- `ui_prompt_summary_hides_secret_like_layer_preview`
- `ui_prompt_summary_frame_encodes_as_single_line`

- [x] **Step 2: Add runtime normalization test**

Add test named:

- `prompt_snapshot_event_normalizes_to_ui_prompt_summary`

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p ruse-ui-bridge --test prompt_summary
cargo test -p ruse-ui-bridge --test bridge_snapshots
```

Expected: tests fail because prompt summary bridge contracts are not implemented.

### Task 3: Implement UI Prompt Summary

**Files:**
- Modify: `crates/ruse-ui-bridge/src/event.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add UI summary contracts**

Implement:

- `UiPromptSummary`
- `UiPromptLayerSummary`
- `UiPromptDiagnosticSummary`

- [x] **Step 2: Add UI payload or frame variant**

Add a prompt summary variant that can be emitted as a UI frame.

- [x] **Step 3: Normalize prompt snapshot events**

Map `PromptSnapshotRecorded` runtime events into the UI prompt summary payload.

- [x] **Step 4: Enforce redaction**

Ensure hidden and secret-like summaries never include prompt content previews.

### Task 4: Verify And Complete M16

**Files:**
- Modify: `docs/plans/M16-05-ui-prompt-summary.plan.md`
- Modify: `docs/plans/M16-prompt-runtime-dynamic-management.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test prompt_summary
cargo test -p ruse-ui-bridge --test bridge_snapshots
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: formatting, clippy, and workspace tests pass.

- [x] **Step 3: Mark M16 complete**

Update M16 milestone status and active focus to M17.

- [ ] **Step 4: Commit**

Run:

```bash
git add crates/ruse-ui-bridge docs README.md
git commit -m "feat: complete m16 ui prompt summary"
```

Expected: commit succeeds if this slice is landed independently.
