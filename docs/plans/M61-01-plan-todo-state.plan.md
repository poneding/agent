# M61-01 Shared Plan/Todo State Contracts Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add shared TodoWrite-compatible plan/todo contracts and validation helpers in runtime crates.

**Architecture:** `agent-types` owns serde contracts and event payload additions. `agent-core` owns validation, update application, progress summaries, and redaction. Presentation crates consume these contracts only in later M61 slices.

**Tech Stack:** Rust 2024 edition, serde, existing typed ids/timestamps/task state, existing task text redaction behavior, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M61-01-plan-todo-state.spec.md`
- Read: `docs/specs/M61-plan-todo-task-ux.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-types/src/ids.rs`
- Read: `crates/agent-types/src/state.rs`
- Read: `crates/agent-types/src/event.rs`
- Read: `crates/agent-core/src/task_review.rs`
- Read: `crates/agent-scheduler/src/command.rs`

- [x] **Step 1: Review the contract home**

M61 planning identified `agent-types` as the right home for durable plan/todo contracts and `agent-core` as the right home for validation and update helpers.

- [ ] **Step 2: Run pre-implementation focused checks**

Run:

```bash
cargo test -p agent-types
cargo test -p agent-core task_review
```

Expected: existing shared type and task review tests pass before adding plan/todo contracts.

### Task 2: Write Failing Contract Tests

**Files:**
- Modify or create: `crates/agent-types/src/plan.rs`
- Modify: `crates/agent-types/src/lib.rs`
- Modify if adding events: `crates/agent-types/src/event.rs`
- Create: `crates/agent-core/tests/plan_todo_state.rs`

- [ ] **Step 1: Add serde and validation expectations**

Cover:

- todo status, priority, source, and evidence kind serialize as snake_case
- a valid snapshot with pending, in-progress, completed, and cancelled items validates
- progress counts match item states
- empty content, empty reasons, duplicate item ids, and multiple in-progress items fail
- secret-like text is redacted from content, evidence summaries, diagnostics, and reasons
- stale revision updates fail
- successful updates increment revision

- [ ] **Step 2: Run focused tests and verify expected failures**

Run:

```bash
cargo test -p agent-types plan
cargo test -p agent-core --test plan_todo_state
```

Expected: new tests fail until the contracts and validation helpers are implemented.

### Task 3: Implement `agent-types` Contracts

**Files:**
- Create: `crates/agent-types/src/plan.rs`
- Modify: `crates/agent-types/src/lib.rs`
- Modify if needed: `crates/agent-types/src/ids.rs`
- Modify if needed: `crates/agent-types/src/event.rs`

- [ ] **Step 1: Add typed plan/todo structures**

Implement ids, enums, item snapshots, evidence, diagnostics, progress, and snapshot structs with serde derives and bounded public shapes.

- [ ] **Step 2: Add event payloads only if needed by tests**

If this slice persists plan/todo changes as runtime events, add typed event payload variants with serde tests. If event emission is deferred to a later slice, document the deferral in the plan checklist before commit.

- [ ] **Step 3: Run type-focused tests**

Run:

```bash
cargo test -p agent-types plan
```

Expected: type serialization tests pass.

### Task 4: Implement `agent-core` Validation and Updates

**Files:**
- Create: `crates/agent-core/src/plan.rs`
- Modify: `crates/agent-core/src/lib.rs`
- Create: `crates/agent-core/tests/plan_todo_state.rs`

- [ ] **Step 1: Add snapshot validation**

Validate non-empty fields, duplicate ids, todo state invariants, bounded/redacted text, and progress counts.

- [ ] **Step 2: Add update application**

Apply replace-all, status update, and evidence append operations with revision checks and typed errors.

- [ ] **Step 3: Keep side effects out**

Do not add tool execution, scheduler mutation, CLI parsing, Desktop state, or TypeScript logic in this slice.

- [ ] **Step 4: Run core-focused tests**

Run:

```bash
cargo test -p agent-core --test plan_todo_state
```

Expected: validation, redaction, progress, and update tests pass.

### Task 5: Verify and Commit

**Files:**
- Modify: `docs/plans/M61-01-plan-todo-state.plan.md`
- Modify if implementation status changes: `docs/milestones/README.md`

- [ ] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-types -p agent-core --all-targets -- -D warnings
cargo test -p agent-types
cargo test -p agent-core --test plan_todo_state
```

Expected: focused checks pass.

- [ ] **Step 2: Stage exact paths and commit**

Run exact-path staging for M61-01 source, tests, and docs, then:

```bash
git commit -m "feat: add m61 plan todo state contracts"
```

Expected: commit contains only M61-01 source, tests, and relevant docs/status updates.

### Completion Checklist

- [ ] Plan/todo contracts live in shared runtime crates.
- [ ] Contracts are serde-friendly and covered by serialization tests.
- [ ] Validation rejects empty text, duplicate ids, stale revisions, and invalid in-progress state.
- [ ] Progress summaries and evidence links are typed and tested.
- [ ] Text normalization is bounded and redacted.
- [ ] Focused fmt, clippy, and tests pass.
