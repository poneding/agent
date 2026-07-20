# M16 Prompt Runtime and Dynamic Prompt Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a versioned dynamic prompt runtime with profiles, overlays, source diagnostics, and per-turn prompt snapshots.

**Architecture:** Treat this file as the M16 coordination plan. `ruse-context` owns prompt contracts and assembly, `ruse-core` records snapshots and diagnostics, and UI crates render summaries through `ruse-ui-bridge`.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde contracts, SQLite-backed event/checkpoint persistence, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M16 Technical Design

**Files:**
- Create: `docs/specs/M16-00-prompt-runtime-technical-design.spec.md`
- Create: `docs/plans/M16-00-prompt-runtime-technical-design.plan.md`

- [x] **Step 1: Write the M16 technical design baseline**

Define prompt profile schema, layer sources, overlay rules, snapshot persistence, UI-safe diagnostics, secret redaction, and replay requirements.

- [x] **Step 2: Write the M16 design execution plan**

Create the technical-design plan with focused documentation verification and commit steps.

### Task 2: Decompose M16 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M16-01-prompt-profile-contracts.spec.md`
- Create: `docs/plans/M16-01-prompt-profile-contracts.plan.md`
- Create: `docs/specs/M16-02-prompt-overlays-diagnostics.spec.md`
- Create: `docs/plans/M16-02-prompt-overlays-diagnostics.plan.md`
- Create: `docs/specs/M16-03-prompt-snapshot-persistence.spec.md`
- Create: `docs/plans/M16-03-prompt-snapshot-persistence.plan.md`
- Create: `docs/specs/M16-04-cli-prompt-inspection.spec.md`
- Create: `docs/plans/M16-04-cli-prompt-inspection.plan.md`
- Create: `docs/specs/M16-05-ui-prompt-summary.spec.md`
- Create: `docs/plans/M16-05-ui-prompt-summary.plan.md`

- [x] **Step 1: Create slice specs**

Each slice spec defines public contracts, state changes, redaction behavior, tests, and acceptance criteria.

- [x] **Step 2: Create slice plans**

Each slice plan starts with failing tests, exact files, focused commands, workspace gates, and commit boundaries.

### Task 3: Implement M16 Slices in Order

**Files:**
- Execute: `docs/plans/M16-01-prompt-profile-contracts.plan.md`
- Execute: `docs/plans/M16-02-prompt-overlays-diagnostics.plan.md`
- Execute: `docs/plans/M16-03-prompt-snapshot-persistence.plan.md`
- Execute: `docs/plans/M16-04-cli-prompt-inspection.plan.md`
- Execute: `docs/plans/M16-05-ui-prompt-summary.plan.md`

- [x] **Step 1: Implement prompt profile contracts**

Add typed profile, layer, source, visibility, sensitivity, and version contracts in `ruse-context`.

- [x] **Step 2: Implement overlays and diagnostics**

Add deterministic ordering, replacement rules, duplicate diagnostics, and budget integration.

- [x] **Step 3: Persist prompt snapshots**

Record prompt profile ids, layer hashes, and prompt snapshot metadata in configured turns.

- [x] **Step 4: Add CLI prompt inspection**

Expose UI-safe prompt diagnostics without hidden or sensitive content.

- [x] **Step 5: Add UI bridge prompt summaries**

Expose prompt summaries for TUI and Desktop clients through stable frames.

### Task 4: Complete M16

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M16-prompt-runtime-dynamic-management.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: formatting, clippy, and workspace tests pass.

- [x] **Step 2: Mark M16 complete**

Update milestone status and advance active focus to M17.

- [ ] **Step 3: Commit M16 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m16 prompt runtime"
```

Expected: commit succeeds with source, tests, and docs together.
