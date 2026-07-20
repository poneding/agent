# M08-04 Notifications Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add trace-safe task notification contracts, runtime events, and UI bridge normalization.

**Architecture:** `ruse-scheduler` owns notification contracts, `ruse-types` owns durable runtime payloads, and `ruse-ui-bridge` owns UI-safe summaries.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `ruse-scheduler`, `ruse-types`, `ruse-ui-bridge`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M08-04-notifications.spec.md`
- Read: `crates/ruse-types/src/event.rs`
- Read: `crates/ruse-ui-bridge/src/event.rs`

- [x] **Step 1: Review the spec**

```bash
sed -n '1,280p' docs/specs/M08-04-notifications.spec.md
```

### Task 2: Write Notification RED Tests

**Files:**
- Create: `crates/ruse-scheduler/tests/notifications.rs`
- Create: `crates/ruse-types/tests/m08_task_notifications.rs`
- Create: `crates/ruse-ui-bridge/tests/m08_task_notifications.rs`

- [x] **Step 1: Add failing tests**

Cover:

- notification validation and in-memory sink behavior
- runtime event serialization for task notifications
- UI bridge normalization without raw prompt/tool content
- waiting-approval and completion notification shapes

- [x] **Step 2: Verify RED**

```bash
cargo test --workspace m08_task_notifications
cargo test -p ruse-scheduler notifications
```

Expected: fails because notification contracts do not exist.

### Task 3: Implement Notifications

**Files:**
- Create: `crates/ruse-scheduler/src/notification.rs`
- Create/Modify: `crates/ruse-types/src/task.rs`
- Modify: `crates/ruse-types/src/event.rs`
- Modify: `crates/ruse-types/src/lib.rs`
- Modify: `crates/ruse-ui-bridge/src/event.rs`

- [x] **Step 1: Add scheduler notification contracts**

Implement `TaskNotificationKind`, `TaskNotification`, and `TaskNotificationSink`.

- [x] **Step 2: Add runtime event payload**

Implement `TaskNotificationRecorded` and add a `RuntimeEventPayload` variant.

- [x] **Step 3: Add UI normalization**

Map task notification events to UI-safe summaries.

- [x] **Step 4: Verify GREEN**

```bash
cargo test --workspace m08_task_notifications
cargo test -p ruse-scheduler notifications
```

### Task 4: Complete M08

**Files:**
- Modify: `docs/plans/M08-04-notifications.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Mark M08 complete**

Set active focus to M09 planning.

- [x] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m08 scheduler background tasks"
```
