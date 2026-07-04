# M62-05 Crash Smoke, Docs, and Closeout Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic crash/restart recovery smoke coverage, document CLI/Desktop recovery workflows, integrate closeout evidence, and mark M62 complete only after final gates pass.

**Architecture:** `agent-core` owns crash/restart recovery fixture behavior. `agent-eval` may expose the fixture through the default credential-free eval suite. CLI/Desktop docs describe typed runtime/bridge recovery surfaces without implementing runtime logic in presentation layers.

**Tech Stack:** Rust 2024 edition, serde contracts, existing checkpoint/replay/idempotency stores, `RecoveryEngine`, `CommandRetryPlanner`, conflict recovery summaries, `agent-eval` manifests/suites, Markdown docs, Cargo fmt/clippy/test, Vitest, Vite build, `cargo xtask eval`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M62-05-crash-smoke-closeout.spec.md`
- Read: `docs/specs/M62-reliability-recovery-productization.spec.md`
- Read: `docs/plans/M62-reliability-recovery-productization.plan.md`
- Read: `docs/specs/M62-01-recovery-snapshot-resume.spec.md`
- Read: `docs/specs/M62-02-cli-json-replay-resume-fork.spec.md`
- Read: `docs/specs/M62-03-idempotent-retry-ux.spec.md`
- Read: `docs/specs/M62-04-conflict-diagnostics.spec.md`
- Read: `crates/agent-core/src/recovery.rs`
- Read: `crates/agent-core/src/command_retry.rs`
- Read: `crates/agent-core/src/conflict_recovery.rs`
- Read: `crates/agent-core/tests/recovery_snapshot.rs`
- Read: `crates/agent-core/tests/conflict_recovery.rs`
- Read: `crates/agent-eval/src/manifest.rs`
- Read: `crates/agent-eval/src/runner.rs`
- Read: `docs/eval/default.suite.json`

- [x] **Step 1: Review M62 recovery contracts**

Identify the exact contract fields needed for crash smoke evidence: checkpoint metadata, replay count, readiness, pending tool calls, idempotent recovery summary, retry plan, conflicts, diagnostics, and redaction.

### Task 2: Add Runtime Crash/Restart Smoke

**Files:**
- Create: `crates/agent-core/tests/crash_resume_smoke.rs`

- [x] **Step 1: Build fixture setup**

Create a deterministic in-memory fixture that records:

- a conversation and turn
- a pre-crash checkpoint
- at least one tail event after the checkpoint
- one pending tool call
- one completed idempotent tool effect
- one retryable failed command/test evidence, if current retry planner inputs support it cleanly
- one worktree conflict summary via `ConflictRecoveryMapper`
- one secret-like input that must not appear in user-facing summaries

- [x] **Step 2: Simulate restart**

Drop the fixture runtime objects and reconstruct recovery using fresh calls into `RecoveryEngine`, idempotency store reads, retry planner, and conflict mapper over the stored fixture data.

- [x] **Step 3: Assert recovery behavior**

Assert:

- recovery snapshot contains checkpoint metadata
- replay tail count is non-zero
- pending tool count is preserved
- idempotent effect appears as reuse/skip evidence
- readiness is blocked when conflicts are attached
- resume dry-run is blocked when readiness is not safe
- retry plan is dry-run and evidence-linked if the fixture includes command retry
- serialized reports do not contain secret-like values

- [x] **Step 4: Run focused runtime test**

Run:

```bash
cargo test -p agent-core --test crash_resume_smoke
```

Expected: crash/restart recovery behavior is deterministic, redacted, and credential-free.

### Task 3: Add Eval or Smoke Gate Integration

**Files:**
- Modify: `crates/agent-eval/src/manifest.rs`
- Modify: `crates/agent-eval/src/runner.rs`
- Modify: `crates/agent-eval/tests/runner.rs`
- Modify: `crates/agent-eval/tests/corpus.rs`
- Modify: `docs/eval/default.suite.json`
- Create: `docs/eval/scenarios/m62-recovery-crash-smoke.eval.json`
- Create or update: `docs/eval/goldens/default-m62-recovery-crash-smoke.golden.json`
- Modify if needed: `xtask/src/lib.rs`

- [x] **Step 1: Decide integration path**

Prefer adding a `recovery_crash_smoke` eval scenario if it can reuse the runtime fixture cleanly. If eval integration would duplicate too much runtime logic, add an explicit `xtask` smoke command that runs the core test/evidence path and document why.

- [x] **Step 2: Implement credential-free eval/smoke**

The smoke must require no provider credentials, network, live shell command side effects, or external workspace mutation.

- [x] **Step 3: Run eval-focused checks**

Run:

```bash
cargo test -p agent-eval
cargo test -p agent-cli --test eval_suite_commands
cargo xtask eval
```

Expected: the default eval or documented smoke gate includes M62 recovery evidence and remains deterministic.

### Task 4: Add Recovery Workflow Docs

**Files:**
- Create: `docs/cli/recovery-workflows.md`
- Create: `docs/desktop/recovery-workflows.md`
- Modify: `README.md`
- Modify if relevant: `crates/agent-cli/tests/daily_driver_recipes.rs`

- [x] **Step 1: Document CLI workflows**

Include:

- recovery status
- resume dry-run
- replay inspect
- replay fork
- command retry dry-run
- conflict inspect
- crash/restart expectations
- redaction and bounded evidence notes

- [x] **Step 2: Document Desktop workflows**

Include:

- recovery readiness
- task conflict summaries
- approval waits
- retryable command/test failures
- replay/fork expectations
- what Desktop does and does not do after restart

- [x] **Step 3: Add docs coverage if available**

If existing tests assert docs content, update or add focused tests for the new docs.

### Task 5: Close M62 Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M62-reliability-recovery-productization.plan.md`
- Modify: `docs/plans/M62-05-crash-smoke-closeout.plan.md`

- [x] **Step 1: Run final closeout gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
```

Expected: all final M62 gates pass.

Result: all final closeout gates passed. `cargo test --workspace` was run with `CARGO_TARGET_DIR=target/m62-gate` after the default target produced a macOS dyld/code-signing stall for one generated test binary; the isolated target ran the same workspace test gate successfully. `cargo xtask eval` reported `total=5 passed=5 failed=0 skipped=0 invalid=0 not_run=0`, including `default-m62-recovery-crash-smoke`.

- [x] **Step 2: Update status docs**

Mark:

- M62-05 spec, plan, and implementation complete
- M62 implementation complete
- roadmap M62 status complete
- next active focus M63 Worktree and Agent Team Workflows

Do not mark M63-M66 complete.

- [x] **Step 3: Commit M62-05**

Exact-stage only M62-05 source, eval, docs, and status files:

```bash
git status --short --branch
git diff --check
git add README.md docs/specs/M62-05-crash-smoke-closeout.spec.md docs/plans/M62-05-crash-smoke-closeout.plan.md docs/plans/M62-reliability-recovery-productization.plan.md docs/cli/recovery-workflows.md docs/desktop/recovery-workflows.md docs/eval/default.suite.json docs/eval/scenarios/m62-recovery-crash-smoke.eval.json docs/eval/goldens/default-m62-recovery-crash-smoke.golden.json docs/milestones/README.md docs/roadmap/PRODUCT_ROADMAP.md crates/agent-core/tests/crash_resume_smoke.rs crates/agent-eval/src/manifest.rs crates/agent-eval/src/runner.rs crates/agent-eval/tests/runner.rs crates/agent-eval/tests/corpus.rs xtask/src/lib.rs
git commit -m "feat: add m62-05 recovery crash smoke closeout"
```

Expected: M62-05 commit contains deterministic recovery crash smoke, docs, eval/smoke integration, closeout status, and verification evidence.
