# M62 Reliability and Recovery Productization Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Productize recovery so interrupted, crashed, conflicted, or retryable coding work can be inspected, resumed, replayed, forked, retried, and audited from shared runtime contracts.

**Architecture:** `ruse-storage` owns durable checkpoint, replay, event, and idempotency stores; `ruse-core` owns recovery snapshots, resume validation, idempotent recovery visibility, and failed-command retry planning; `ruse-worktree` owns conflict reports; `ruse-ui-bridge` owns UI-safe commands/frames; CLI/TUI/Desktop render typed contracts without duplicating recovery logic.

**Tech Stack:** Rust 2024 edition, serde contracts, existing checkpoint/replay/idempotency stores, existing approval resume recovery, worktree conflict reports, UI bridge frames, CLI text/JSON commands, Ratatui task/recovery panels, Desktop Rust/React/Vitest fixtures, Cargo fmt/clippy/test, `npm --prefix apps/desktop test`, `npm --prefix apps/desktop run build`, and `cargo xtask eval`.

---

### Task 1: Confirm M62 Inputs

**Files:**
- Read: `docs/specs/M62-reliability-recovery-productization.spec.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`
- Read: `docs/architecture/README.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/storage.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `docs/specs/M04-checkpoint-recovery-trace.spec.md`
- Read: `docs/specs/M18-ruse-tool-loop-runtime.spec.md`
- Read: `docs/specs/M19-interactive-approval-safe-actions.spec.md`
- Read: `docs/specs/M26-runtime-observability-evidence.spec.md`
- Read: `docs/specs/M56-coding-agent-golden-path-v1.spec.md`
- Read: `crates/ruse-core/src/recovery.rs`
- Read: `crates/ruse-core/src/idempotency.rs`
- Read: `crates/ruse-core/tests/approval_resume_recovery.rs`
- Read: `crates/ruse-storage/src/checkpoint.rs`
- Read: `crates/ruse-storage/src/replay.rs`
- Read: `crates/ruse-storage/src/idempotency.rs`
- Read: `crates/ruse-worktree/src/conflict.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-ui-bridge/src/event.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-tui/src/task_panel.rs`
- Read: `crates/ruse-desktop/src/task.rs`
- Read: `apps/desktop/src/types.ts`

- [x] **Step 1: Review existing recovery foundations**

Baseline foundations identified during M62 planning:

- `ruse-core::RecoveryEngine` plans from latest checkpoint or sequence 1, replays the tail with `ReplayCursor`, filters pending tool call ids, and rejects corrupt checkpoint state.
- `ruse-core::IdempotentToolExecutor` records completed tool effects and replays recorded output without duplicating side effects.
- `ruse-storage` owns checkpoint, replay cursor, event, and idempotency stores with in-memory and SQLite coverage.
- `ruse-worktree::detect_conflicts` produces typed relative-path conflict reports.
- Approval resume recovery already reconstructs pending approval waits and idempotent tool outcomes after restart.
- UI bridge currently maps checkpoint/replay runtime events to generic progress/checkpoint frames but does not expose product-level recovery snapshots.

- [ ] **Step 2: Run baseline focused checks before implementation**

Run:

```bash
cargo test -p ruse-core --test approval_resume_recovery
cargo test -p ruse-core --test comprehensive_turn_idempotency
cargo test -p ruse-core --test comprehensive_turn_guards
cargo test -p ruse-storage --test checkpoint_store
cargo test -p ruse-storage --test replay_integration
cargo test -p ruse-worktree --test conflict_detection
cargo test -p ruse-ui-bridge --test task_command_bridge
cargo test -p ruse-cli --test final_review
```

Expected: existing recovery, idempotency, checkpoint, replay, conflict, bridge, and CLI evidence foundations pass before M62 implementation changes.

### Task 2: M62-01 Recovery Snapshot and Resume Contracts

**Files:**
- Create: `docs/specs/M62-01-recovery-snapshot-resume.spec.md`
- Create: `docs/plans/M62-01-recovery-snapshot-resume.plan.md`
- Modify: `crates/ruse-types/src/lib.rs`
- Create: `crates/ruse-types/src/recovery.rs`
- Modify: `crates/ruse-core/src/recovery.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Create: `crates/ruse-core/tests/recovery_snapshot.rs`

- [ ] **Step 1: Plan recovery snapshot slice**

Define typed recovery snapshots, readiness, diagnostics, pending tool calls, idempotent effect summaries, approval waits, checkpoint/replay metadata, and resume dry-run behavior.

- [ ] **Step 2: Implement runtime recovery contracts**

Extend existing recovery planning into serde-friendly product snapshots while preserving the current checkpoint/replay/idempotency behavior.

- [ ] **Step 3: Verify recovery snapshot behavior**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-types -p ruse-core --all-targets -- -D warnings
cargo test -p ruse-types
cargo test -p ruse-core --test recovery_snapshot
cargo test -p ruse-core --test approval_resume_recovery
```

Expected: snapshots are typed, redacted, deterministic, and resume readiness handles clean, pending, approval-wait, idempotent, and corrupt checkpoint cases.

### Task 3: M62-02 CLI/JSON Replay, Resume, and Fork Commands

**Files:**
- Create: `docs/specs/M62-02-cli-json-replay-resume-fork.spec.md`
- Create: `docs/plans/M62-02-cli-json-replay-resume-fork.plan.md`
- Modify: `crates/ruse-ui-bridge/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Create: `crates/ruse-ui-bridge/tests/recovery_bridge.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Create: `crates/ruse-cli/tests/recovery_commands.rs`

- [ ] **Step 1: Plan command slice**

Define `recovery status`, `recovery resume`, `replay inspect`, and `replay fork` text/JSON shapes, bridge command envelopes, stable summaries, and error behavior.

- [ ] **Step 2: Implement CLI/JSON commands**

Expose recovery snapshots and replay/fork outcomes through shared bridge/runtime contracts. Keep fork creation explicit and durable.

- [ ] **Step 3: Verify command behavior**

Run:

```bash
cargo clippy -p ruse-ui-bridge -p ruse-cli --all-targets -- -D warnings
cargo test -p ruse-ui-bridge --test recovery_bridge
cargo test -p ruse-cli --test recovery_commands
```

Expected: text and JSON recovery commands are stable, bounded, redacted, and evidence-linked.

### Task 4: M62-03 Idempotent Recovery and Failed Command Retry UX

**Files:**
- Create: `docs/specs/M62-03-idempotent-retry-ux.spec.md`
- Create: `docs/plans/M62-03-idempotent-retry-ux.plan.md`
- Modify: `crates/ruse-core/src/idempotency.rs`
- Create: `crates/ruse-core/src/command_retry.rs`
- Modify: `crates/ruse-core/src/recovery.rs`
- Create: `crates/ruse-core/tests/failed_command_retry.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/tests/recovery_commands.rs`

- [ ] **Step 1: Plan idempotent/retry slice**

Define safe summaries for replayed tool effects, skipped side effects, pending tool calls, retryable command/test failures, retry requests, and attempt linkage.

- [ ] **Step 2: Implement recovery visibility and retry planning**

Thread idempotent effect summaries and failed-command retry plans into recovery snapshots and CLI/bridge surfaces. Retried commands must still go through policy and command/test execution paths.

- [ ] **Step 3: Verify idempotency and retry behavior**

Run:

```bash
cargo clippy -p ruse-core -p ruse-ui-bridge -p ruse-cli --all-targets -- -D warnings
cargo test -p ruse-core --test comprehensive_turn_idempotency
cargo test -p ruse-core --test failed_command_retry
cargo test -p ruse-cli --test recovery_commands
```

Expected: completed side effects are visibly replayed/skipped, retryable command failures produce explicit retry plans, and conflicts or stale targets block safely.

### Task 5: M62-04 Conflict Diagnostics and Recovery Surfacing

**Files:**
- Create: `docs/specs/M62-04-conflict-diagnostics.spec.md`
- Create: `docs/plans/M62-04-conflict-diagnostics.plan.md`
- Modify: `crates/ruse-worktree/src/conflict.rs`
- Modify: `crates/ruse-core/src/recovery.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-tui/src/task_panel.rs`
- Modify: `crates/ruse-desktop/src/task.rs`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/state/tasks.ts`
- Modify: `apps/desktop/src/components/TaskDetail.tsx`
- Create: `crates/ruse-desktop/tests/recovery_view.rs`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`

- [x] **Step 1: Plan conflict surface slice**

Define conflict summary contracts, recovery readiness interaction, TUI/Desktop presentation fixtures, and safe next-action text.

- [x] **Step 2: Implement conflict diagnostics**

Expose conflict reports through recovery snapshots and UI-safe task/recovery surfaces without embedding raw file contents.

- [x] **Step 3: Verify conflict diagnostics**

Run:

```bash
cargo clippy -p ruse-worktree -p ruse-core -p ruse-ui-bridge -p ruse-tui -p ruse-desktop --all-targets -- -D warnings
cargo test -p ruse-worktree --test conflict_detection
cargo test -p ruse-tui --test recovery_panel
cargo test -p ruse-desktop --test recovery_view
npm --prefix apps/desktop test -- desktop-smoke
```

Expected: conflicts are typed, redacted, actionable, and visible in shared CLI/Desktop-facing recovery surfaces.

### Task 6: M62-05 Crash Smoke, Docs, and Closeout

**Files:**
- Create: `docs/specs/M62-05-crash-smoke-closeout.spec.md`
- Create: `docs/plans/M62-05-crash-smoke-closeout.plan.md`
- Modify: `docs/eval/default.suite.json`
- Create or modify: `docs/eval/scenarios/`
- Modify: `crates/ruse-eval/`
- Modify: `xtask/src/main.rs`
- Create: `docs/cli/recovery-workflows.md`
- Create: `docs/desktop/recovery-workflows.md`
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Plan crash smoke and closeout slice**

Define deterministic crash/restart fixture, eval or xtask command integration, docs, and closeout evidence requirements.

- [x] **Step 2: Implement crash smoke and docs**

Add a credential-free recovery smoke that proves checkpoint resume, idempotent skip/replay, retry evidence, and recovery reporting after a simulated interruption.

- [x] **Step 3: Run final M62 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
```

Expected: M62 recovery workflows are implemented, documented, smoke-tested, and do not regress earlier product milestones.

Result: final M62 gates passed. `cargo test --workspace` was run with `CARGO_TARGET_DIR=target/m62-gate` after the default target produced a macOS dyld/code-signing stall for one generated test binary; the isolated target ran the same workspace test gate successfully. The `xtask` quality-gate fixture printed the expected fake rustc flag message and exited successfully.

### Task 7: Commit Discipline

**Files:**
- Modify/create only files for the active slice and required status docs.

- [ ] **Step 1: Commit milestone planning**

After this milestone plan and spec are created, run:

```bash
git status --short --branch
git diff --check
git add docs/specs/M62-reliability-recovery-productization.spec.md docs/plans/M62-reliability-recovery-productization.plan.md docs/milestones/README.md docs/roadmap/PRODUCT_ROADMAP.md README.md
git commit -m "docs: add m62 recovery productization plan"
```

Expected: planning docs are committed before implementation slices begin.

- [x] **Step 2: Commit each implementation slice**

For each M62 implementation slice, exact-stage the slice spec, slice plan, code, tests, and docs touched by that slice only, then commit with a focused message.
