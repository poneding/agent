# M56-04 Final Review Summary Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a typed final review contract that summarizes changed files, command/test evidence, trace/audit evidence, unresolved risks, next actions, diagnostics, and replay reconstruction status for the M56 coding-agent workflow.

**Architecture:** `agent-core` owns review construction and replay reconstruction. `agent-cli` renders text/JSON. `agent-ui-bridge` owns safe final review snapshots. Runtime logic must not move into UI crates.

**Tech Stack:** Rust 2024 edition, serde, existing `LocalCodingRun`, `EditDiffFlowReport`, `CommandTestLoopReport`, runtime events, observability/trace/audit helpers, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M56-04-final-review-summary.spec.md`
- Read: `docs/specs/M56-coding-agent-golden-path-v1.spec.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `crates/agent-core/src/local_coding.rs`
- Read: `crates/agent-core/src/edit_diff.rs`
- Read: `crates/agent-core/src/command_test.rs`
- Read: `crates/agent-core/src/observability.rs`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`

- [ ] **Step 1: Review final review boundaries**

Run:

```bash
sed -n '1,260p' docs/specs/M56-04-final-review-summary.spec.md
```

Expected: scope is limited to final review construction and presentation. Full golden-path E2E remains M56-05.

### Task 2: Write Failing Core Tests

**Files:**
- Create: `crates/agent-core/tests/final_review.rs`

- [ ] **Step 1: Add core tests**

Cover:

- final review composes changed files from `EditDiffFlowReport`
- final review composes command/test evidence from `CommandTestLoopReport`
- final review reconstructs changed files, command results, final summary, trace count, audit count, and replay status from local coding events
- missing changed-file evidence produces a diagnostic and unresolved risk
- missing command/test evidence produces a diagnostic and unresolved risk
- failed command/test evidence produces an unresolved risk and next action
- secret-like summaries, diagnostics, risks, and next actions are redacted

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
cargo test -p agent-core final_review
```

Expected: tests fail before the core final review module exists.

### Task 3: Implement Core Final Review

**Files:**
- Create: `crates/agent-core/src/final_review.rs`
- Modify: `crates/agent-core/src/lib.rs`
- Modify if needed: `crates/agent-core/src/local_coding.rs`

- [ ] **Step 1: Add typed contracts**

Implement `CodingFinalReview`, `FinalReviewInput`, `FinalReviewCommandResult`, diagnostics, risks, next actions, and errors.

- [ ] **Step 2: Compose from reports**

Accept optional M56-02 edit report and M56-03 command/test report and produce a review without executing tools.

- [ ] **Step 3: Reconstruct from events**

Use local coding event patterns, trace/audit counts, and replay reconstruction evidence to build or diagnose a final review from persisted events.

- [ ] **Step 4: Run core checks**

Run:

```bash
cargo test -p agent-core final_review
cargo test -p agent-core local_coding_e2e
```

Expected: core final review and existing local coding tests pass.

### Task 4: Add UI Bridge Snapshot

**Files:**
- Create: `crates/agent-ui-bridge/tests/final_review_snapshot.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`

- [ ] **Step 1: Add bridge snapshot tests**

Cover safe serialization, redaction, bounded lists, risk/next-action fields, replay status, and frame encoding.

- [ ] **Step 2: Implement bridge snapshot**

Expose a UI-safe final review snapshot derived from `CodingFinalReview`.

- [ ] **Step 3: Run bridge checks**

Run:

```bash
cargo test -p agent-ui-bridge final_review_snapshot
```

Expected: UI final review snapshot tests pass.

### Task 5: Add CLI Text and JSON Surface

**Files:**
- Create: `crates/agent-cli/tests/final_review.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Modify if needed: `crates/agent-cli/tests/local_coding.rs`

- [ ] **Step 1: Add CLI tests**

Cover:

- `repo final-review --fixture local-coding` text output includes changed files, commands/tests, risks, next actions, trace/audit evidence, and replay status
- `--json repo final-review --fixture local-coding` emits final review snapshot and done frame
- `--local-summary` remains stable or is intentionally updated with final review-backed output
- secret-like review content is redacted

- [ ] **Step 2: Implement CLI command**

Add parser and renderer using the core final review contract. Keep fixture mode deterministic and side-effect-free beyond the existing local coding fixture setup.

- [ ] **Step 3: Run CLI checks**

Run:

```bash
cargo test -p agent-cli final_review
cargo test -p agent-cli local_coding
```

Expected: final review CLI tests and existing local coding summary tests pass.

### Task 6: Verify and Commit

**Files:**
- Modify: `docs/plans/M56-04-final-review-summary.plan.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-core -p agent-cli -p agent-ui-bridge --all-targets -- -D warnings
cargo test -p agent-core final_review
cargo test -p agent-core local_coding_e2e
cargo test -p agent-cli final_review
cargo test -p agent-cli local_coding
cargo test -p agent-ui-bridge final_review_snapshot
```

Expected: focused checks pass.

- [ ] **Step 2: Commit**

Run exact-path staging for M56-04 source, tests, and docs, then:

```bash
git commit -m "feat: add m56 final review summary"
```

Expected: commit contains only M56-04 source, tests, and documentation/status updates.

### Completion Checklist

- [ ] Core final review contract exists.
- [ ] Review summarizes changed files, command/test evidence, trace/audit evidence, risks, next actions, diagnostics, and replay status.
- [ ] Review can reconstruct from local coding events or report missing evidence.
- [ ] CLI text and JSON surfaces exist.
- [ ] UI bridge final review snapshot is safe and serializable.
- [ ] Existing local coding summary remains stable or is deliberately migrated.
- [ ] Focused fmt/clippy/tests pass.
