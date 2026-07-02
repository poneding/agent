# M62-01 Recovery Snapshot and Resume Contracts Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed recovery snapshots and resume dry-run reports over existing checkpoint, replay, idempotency, and approval-resume foundations.

**Architecture:** `agent-types` owns serde recovery contracts. `agent-core` builds recovery snapshots and dry-run reports from `agent-storage` traits. `agent-storage` remains the owner of checkpoint, replay, event, and idempotency stores. UI crates are not part of this slice except for additive bridge compatibility if needed by compile/tests.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `RecoveryEngine`, `CheckpointStore`, `ReplayCursor`, `IdempotencyStore`, runtime events, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs and Baseline

**Files:**
- Read: `docs/specs/M62-01-recovery-snapshot-resume.spec.md`
- Read: `docs/specs/M62-reliability-recovery-productization.spec.md`
- Read: `docs/plans/M62-reliability-recovery-productization.plan.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/storage.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-core/src/recovery.rs`
- Read: `crates/agent-core/src/idempotency.rs`
- Read: `crates/agent-core/tests/approval_resume_recovery.rs`
- Read: `crates/agent-storage/src/checkpoint.rs`
- Read: `crates/agent-storage/src/replay.rs`
- Read: `crates/agent-storage/src/idempotency.rs`
- Read: `crates/agent-types/src/event.rs`
- Read: `crates/agent-types/src/lib.rs`

- [ ] **Step 1: Run baseline focused checks**

Run:

```bash
cargo test -p agent-core --test approval_resume_recovery
cargo test -p agent-core --test comprehensive_turn_idempotency
cargo test -p agent-core --test comprehensive_turn_guards
cargo test -p agent-storage --test checkpoint_store
cargo test -p agent-storage --test replay_integration
cargo test -p agent-worktree --test conflict_detection
```

Expected: existing recovery, idempotency, guard, checkpoint, replay, and conflict foundations pass before implementation.

### Task 2: Add Typed Recovery Contracts

**Files:**
- Create: `crates/agent-types/src/recovery.rs`
- Modify: `crates/agent-types/src/lib.rs`
- Optionally modify: `crates/agent-types/src/event.rs` only if recovery events become necessary for compile or serde coverage

- [ ] **Step 1: Define snapshot contracts**

Add serde-friendly contracts:

- `RecoverySnapshot`
- `RecoveryCheckpointSummary`
- `RecoveryReplaySummary`
- `RecoveryReadiness`
- `RecoveryDiagnostic`
- `RecoveryDiagnosticSeverity`
- `RecoveredToolCall`
- `RecoveredToolCallStatus`
- `RecoveredToolEffect`
- `RecoveredApprovalWait`
- `ResumeDryRunReport`

Requirements:

- Derive `Clone`, `Debug`, `Eq`, `PartialEq`, `Serialize`, and `Deserialize`.
- Use existing ID types: `ConversationId`, `TurnId`, `CheckpointId`, `EventSeq`, and `Timestamp`.
- Use `serde(rename_all = "snake_case")` for public enums.
- Avoid raw checkpoint JSON, raw tool input, raw output, or secret-bearing fields.

- [ ] **Step 2: Re-export contracts**

Update `agent-types/src/lib.rs` so downstream crates can use the new recovery contracts.

### Task 3: Write Focused Tests First

**Files:**
- Create: `crates/agent-core/tests/recovery_snapshot.rs`

- [ ] **Step 1: Add snapshot tests**

Add tests for:

- no checkpoint starts replay at sequence 1
- checkpoint summary starts replay at next event sequence
- corrupt checkpoint returns a snapshot/dry-run diagnostic rather than falling back
- pending tool calls exclude terminal tool events
- idempotent records remove pending calls and produce effect summaries
- pending approval evidence produces `NeedsApproval` readiness when event evidence exists
- dry-run report is read-only and reports replay/resume/reuse counts

- [ ] **Step 2: Run tests and confirm expected failures before implementation**

Run:

```bash
cargo test -p agent-core --test recovery_snapshot
```

Expected: tests fail because contracts and snapshot APIs do not exist yet.

### Task 4: Implement Snapshot Construction and Dry-Run

**Files:**
- Modify: `crates/agent-core/src/recovery.rs`
- Modify: `crates/agent-core/src/lib.rs` if re-export changes are needed

- [ ] **Step 1: Add recovery snapshot API**

Extend `RecoveryEngine` with APIs equivalent to:

```rust
pub fn snapshot<E, I>(
    &self,
    events: &E,
    idempotency: &I,
    conversation_id: ConversationId,
    turn_id: Option<TurnId>,
) -> RecoverySnapshot
where
    E: EventStore + CheckpointStore,
    I: IdempotencyStore + ?Sized;

pub fn resume_dry_run<E, I>(
    &self,
    events: &E,
    idempotency: &I,
    conversation_id: ConversationId,
    turn_id: Option<TurnId>,
) -> ResumeDryRunReport
where
    E: EventStore + CheckpointStore,
    I: IdempotencyStore + ?Sized;
```

The exact return type may be `Result<_, RecoveryError>` if it better fits existing error style, but corrupt checkpoint and recoverable storage/idempotency failures must be expressible as typed diagnostics.

- [ ] **Step 2: Map checkpoint and replay metadata**

Reuse `latest_checkpoint` and `ReplayCursor::from_sequence` to populate checkpoint and replay summaries.

- [ ] **Step 3: Map pending tool calls and idempotent effects**

Scan replayed events:

- collect proposed tool calls and safe tool names
- collect terminal call ids from completed/failed/observation events
- for remaining proposed calls, check `IdempotencyStore` by idempotency key
- if an effect exists, emit `RecoveredToolEffect`
- otherwise emit `RecoveredToolCall`

- [ ] **Step 4: Map approval waits where typed event evidence exists**

Use existing runtime event payloads for approval requests/waits if available. If the current event model does not carry enough approval detail for a safe summary, emit no invented wait and add a bounded diagnostic only when a pending state is detectable.

- [ ] **Step 5: Compute readiness**

Readiness rules:

- `CorruptCheckpoint` if checkpoint validation fails
- `StorageError` or `Blocked` if required storage/idempotency reads fail
- `NeedsApproval` if pending approval waits exist
- `PendingToolCalls` if pending tool calls exist
- `Ready` if none of the above apply

- [ ] **Step 6: Implement dry-run counts**

Dry-run report should derive counts from the snapshot and perform no side effects.

### Task 5: Verification

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [ ] **Step 1: Run focused formatting, lint, and tests**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-types -p agent-core -p agent-storage -p agent-ui-bridge --all-targets -- -D warnings
cargo test -p agent-types
cargo test -p agent-core --test recovery_snapshot
cargo test -p agent-core --test approval_resume_recovery
cargo test -p agent-core --test comprehensive_turn_idempotency
cargo test -p agent-storage --test checkpoint_store
cargo test -p agent-storage --test replay_integration
```

Expected: all focused checks pass.

### Task 6: Status Docs and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Optionally modify: `docs/roadmap/PRODUCT_ROADMAP.md` if slice status summaries need refreshing

- [ ] **Step 1: Update status docs**

Mark M62-01 slice spec/plan complete when docs are committed. Mark implementation complete only after code/tests pass.

- [ ] **Step 2: Commit planning docs first**

Before code changes, run:

```bash
git status --short --branch
git diff --check
git add docs/specs/M62-01-recovery-snapshot-resume.spec.md docs/plans/M62-01-recovery-snapshot-resume.plan.md docs/milestones/README.md
git commit -m "docs: add m62-01 recovery snapshot plan"
```

Expected: M62-01 slice planning is committed before implementation.

- [ ] **Step 3: Commit implementation after verification**

After focused verification passes, exact-stage only M62-01 code, tests, and status docs, then commit:

```bash
git status --short --branch
git diff --check
git add crates/agent-types/src/recovery.rs crates/agent-types/src/lib.rs crates/agent-core/src/recovery.rs crates/agent-core/src/lib.rs crates/agent-core/tests/recovery_snapshot.rs docs/milestones/README.md
git commit -m "feat: add m62-01 recovery snapshots"
```

Expected: implementation commit contains only the M62-01 runtime contracts, tests, and status docs.
