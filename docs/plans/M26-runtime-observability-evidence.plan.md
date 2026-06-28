# M26 Runtime Observability Evidence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a durable, redacted, replayable observability evidence layer shared by CLI, TUI, Desktop, and support workflows.

**Architecture:** `agent-telemetry` exports provider-neutral telemetry records, `agent-core` composes runtime evidence from events/audit/telemetry/replay, and UI crates only render typed summaries.

**Tech Stack:** Rust 2024 edition, serde, typed runtime events, audit export, telemetry records, storage replay, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M26 Technical Design

**Files:**
- Create: `docs/specs/M26-00-runtime-observability-technical-design.spec.md`
- Create: `docs/plans/M26-00-runtime-observability-technical-design.plan.md`

- [x] **Step 1: Write the M26 technical design baseline**

Define evidence bundle ownership, privacy rules, correlation keys, replay semantics, CLI/Desktop surfaces, and verification gates.

### Task 2: Decompose M26 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M26-01-telemetry-export-bundles.spec.md`
- Create: `docs/plans/M26-01-telemetry-export-bundles.plan.md`
- Create: `docs/specs/M26-02-runtime-evidence-builder.spec.md`
- Create: `docs/plans/M26-02-runtime-evidence-builder.plan.md`
- Create: `docs/specs/M26-03-cli-observability-commands.spec.md`
- Create: `docs/plans/M26-03-cli-observability-commands.plan.md`
- Create: `docs/specs/M26-04-ui-observability-snapshots.spec.md`
- Create: `docs/plans/M26-04-ui-observability-snapshots.plan.md`
- Create: `docs/specs/M26-05-replay-evidence-verification.spec.md`
- Create: `docs/plans/M26-05-replay-evidence-verification.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines concrete contracts, failure handling, privacy behavior, and tests.

- [x] **Step 2: Create slice plans**

Each plan names files, focused tests, verification commands, and commit points.

### Task 3: Implement M26 Slices in Order

**Files:**
- Execute: `docs/plans/M26-01-telemetry-export-bundles.plan.md`
- Execute: `docs/plans/M26-02-runtime-evidence-builder.plan.md`
- Execute: `docs/plans/M26-03-cli-observability-commands.plan.md`
- Execute: `docs/plans/M26-04-ui-observability-snapshots.plan.md`
- Execute: `docs/plans/M26-05-replay-evidence-verification.plan.md`

- [x] **Step 1: Implement telemetry export bundles**

Add redacted, bounded, filterable telemetry export bundles.

- [ ] **Step 2: Implement runtime evidence builder**

Compose trace entries, audit bundles, telemetry bundles, and replay summaries.

- [ ] **Step 3: Implement CLI observability commands**

Expose text and JSON evidence outputs without raw secret leakage.

- [ ] **Step 4: Implement UI observability snapshots**

Normalize evidence summaries for TUI/Desktop bridge consumers.

- [ ] **Step 5: Implement replay evidence verification**

Prove evidence can be regenerated from event logs and replay fixtures.

### Task 4: Complete M26

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M26-runtime-observability-evidence.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [ ] **Step 2: Mark M26 complete**

Update milestone status and record the next implementation focus.

- [ ] **Step 3: Commit M26 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m26 observability evidence"
```

Expected: commit succeeds.
