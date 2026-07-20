# M62-04 Conflict Diagnostics and Recovery Surfacing Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Surface worktree/file-change conflicts through typed recovery diagnostics and CLI/TUI/Desktop-safe views without implementing merge resolution.

**Architecture:** `ruse-worktree` owns pure conflict detection. `ruse-core` maps conflict reports into recovery-safe summaries and readiness diagnostics. `ruse-ui-bridge` owns UI-safe frames. CLI/TUI/Desktop render typed summaries without recomputing classification.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `ruse-worktree::ConflictReport`, recovery snapshots, UI bridge frames, CLI text/JSON commands, TUI state, Desktop Rust/TypeScript fixtures, Cargo fmt/clippy/test, Vitest.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M62-04-conflict-diagnostics.spec.md`
- Read: `docs/specs/M62-reliability-recovery-productization.spec.md`
- Read: `docs/plans/M62-reliability-recovery-productization.plan.md`
- Read: `docs/specs/M09-02-conflict-detection.spec.md`
- Read: `crates/ruse-worktree/src/conflict.rs`
- Read: `crates/ruse-worktree/tests/conflict_detection.rs`
- Read: `crates/ruse-core/src/recovery.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-tui/src/task_panel.rs`
- Read: `crates/ruse-desktop/src/task.rs`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src/state/tasks.ts`
- Read: `apps/desktop/src/components/TaskDetail.tsx`

- [x] **Step 1: Review conflict and recovery boundaries**

Identify current conflict contracts, path validation behavior, recovery readiness rules, bridge frame patterns, CLI rendering patterns, and TUI/Desktop task/recovery fixture shapes.

### Task 2: Add Recovery Conflict Contracts

**Files:**
- Modify: `crates/ruse-types/src/recovery.rs`
- Create: `crates/ruse-core/src/conflict_recovery.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Modify: `crates/ruse-core/src/recovery.rs`
- Create: `crates/ruse-core/tests/conflict_recovery.rs`
- Modify: `crates/ruse-worktree/src/conflict.rs` only if path/hash redaction or mapping support is missing

- [x] **Step 1: Define typed conflict summaries**

Add serializable contracts equivalent to:

- `RecoveryConflictSummary`
- `RecoveryConflictKind`
- `RecoveryConflictSide`
- `RecoveryConflictNextAction`

Keep paths relative, bounded, sorted, and redacted.

- [x] **Step 2: Map worktree conflicts into recovery conflicts**

Implement a mapper from `ruse_worktree::ConflictReport` to recovery summaries. It must not include raw file contents.

- [x] **Step 3: Integrate readiness**

Extend `RecoverySnapshot` to carry conflict summaries and make active conflicts block automatic resume/readiness with typed diagnostics.

- [x] **Step 4: Add core tests**

Run:

```bash
cargo test -p ruse-worktree --test conflict_detection
cargo test -p ruse-core --test conflict_recovery
```

Expected: conflict mapping is deterministic, safe, sorted, and blocks readiness when conflicts exist.

### Task 3: Add Bridge and CLI Surfaces

**Files:**
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Modify: `crates/ruse-ui-bridge/tests/recovery_bridge.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/src/lib.rs`
- Modify: `crates/ruse-cli/tests/recovery_commands.rs`

- [x] **Step 1: Add conflict bridge frame**

Expose conflict summaries through recovery snapshot frames and, if useful, a dedicated `ConflictInspect` frame.

- [x] **Step 2: Add CLI parser**

Support:

```text
ruse-cli conflict inspect --fixture sample
ruse-cli conflict inspect --conversation <id> [--turn <id>]
```

Use deterministic fixture mode if durable conflict lookup is not available, with an explicit diagnostic.

- [x] **Step 3: Render text and JSON**

Text output should include conflict count, path, kind, side change kinds, hashes if safe, next action, and diagnostics. JSON output should emit typed bridge frames.

- [x] **Step 4: Add tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test recovery_bridge
cargo test -p ruse-cli --test recovery_commands
```

Expected: bridge and CLI output are stable, bounded, sorted, and redacted.

### Task 4: Add TUI/Desktop-Safe Fixtures

**Files:**
- Modify: `crates/ruse-tui/src/task_panel.rs`
- Modify: `crates/ruse-tui/src/trace_panel.rs`
- Modify: `crates/ruse-tui/src/transcript.rs`
- Modify: `crates/ruse-desktop/src/task.rs`
- Create: `crates/ruse-desktop/tests/recovery_view.rs`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/state/tasks.ts`
- Modify: `apps/desktop/src/components/TaskDetail.tsx`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`

- [x] **Step 1: Extend typed models**

Add conflict count/path/kind/next action fields to TUI/Desktop-safe task or recovery summaries.

- [x] **Step 2: Render compact summaries**

Show conflict count and path summaries from typed frames. Do not add merge resolution UI in this slice.

- [x] **Step 3: Add smoke tests**

Run:

```bash
cargo test -p ruse-desktop --test recovery_view
npm --prefix apps/desktop test -- desktop-smoke
```

Expected: conflict summaries render from typed fixtures without raw content.

### Task 5: Verification and Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M62-04-conflict-diagnostics.plan.md`

- [x] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-worktree -p ruse-core -p ruse-ui-bridge -p ruse-cli -p ruse-tui -p ruse-desktop --all-targets -- -D warnings
cargo test -p ruse-worktree --test conflict_detection
cargo test -p ruse-core --test conflict_recovery
cargo test -p ruse-ui-bridge --test recovery_bridge
cargo test -p ruse-cli --test recovery_commands
cargo test -p ruse-desktop --test recovery_view
npm --prefix apps/desktop test -- desktop-smoke
```

Expected: all focused checks pass.

- [x] **Step 2: Update status docs**

Mark M62-04 complete only after typed contracts, readiness integration, bridge/CLI/TUI/Desktop-safe surfaces, tests, and focused checks pass.

- [x] **Step 3: Commit planning docs first**

Before implementation, run:

```bash
git status --short --branch
git diff --check
git add docs/specs/M62-04-conflict-diagnostics.spec.md docs/plans/M62-04-conflict-diagnostics.plan.md docs/milestones/README.md docs/roadmap/PRODUCT_ROADMAP.md
git commit -m "docs: add m62-04 conflict diagnostics plan"
```

Expected: M62-04 slice planning is committed before code changes.

- [x] **Step 4: Commit implementation after verification**

After focused verification passes, exact-stage only M62-04 files and commit:

```bash
git status --short --branch
git diff --check
git add crates/ruse-types/src/recovery.rs crates/ruse-core/src/conflict_recovery.rs crates/ruse-core/src/lib.rs crates/ruse-core/src/recovery.rs crates/ruse-core/tests/conflict_recovery.rs crates/ruse-worktree/src/conflict.rs crates/ruse-ui-bridge/src/frame.rs crates/ruse-ui-bridge/src/lib.rs crates/ruse-ui-bridge/tests/recovery_bridge.rs crates/ruse-cli/src/command.rs crates/ruse-cli/src/lib.rs crates/ruse-cli/tests/recovery_commands.rs crates/ruse-tui/src/task_panel.rs crates/ruse-tui/src/trace_panel.rs crates/ruse-tui/src/transcript.rs crates/ruse-desktop/src/task.rs crates/ruse-desktop/tests/recovery_view.rs apps/desktop/src/types.ts apps/desktop/src/state/tasks.ts apps/desktop/src/components/TaskDetail.tsx apps/desktop/src/__tests__/desktop-smoke.test.tsx docs/milestones/README.md docs/roadmap/PRODUCT_ROADMAP.md docs/plans/M62-04-conflict-diagnostics.plan.md
git commit -m "feat: add m62-04 conflict recovery diagnostics"
```

Expected: implementation commit contains only M62-04 conflict recovery contracts, surfaces, tests, and status docs.
