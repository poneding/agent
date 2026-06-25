# M08-04 Notifications Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add trace-safe task notification contracts, runtime events, and UI bridge normalization.

**Architecture:** `agent-scheduler` owns notification contracts, `agent-types` owns durable runtime payloads, and `agent-ui-bridge` owns UI-safe summaries.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `agent-scheduler`, `agent-types`, `agent-ui-bridge`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M08-04-notifications.spec.md`
- Read: `crates/agent-types/src/event.rs`
- Read: `crates/agent-ui-bridge/src/event.rs`

- [ ] **Step 1: Review the spec**

```bash
sed -n '1,280p' docs/specs/M08-04-notifications.spec.md
```

### Task 2: Write Notification RED Tests

**Files:**
- Create: `crates/agent-scheduler/tests/notifications.rs`
- Create: `crates/agent-types/tests/m08_task_notifications.rs`
- Create: `crates/agent-ui-bridge/tests/m08_task_notifications.rs`

- [ ] **Step 1: Add failing tests**

Cover:

- notification validation and in-memory sink behavior
- runtime event serialization for task notifications
- UI bridge normalization without raw prompt/tool content
- waiting-approval and completion notification shapes

- [ ] **Step 2: Verify RED**

```bash
cargo test --workspace m08_task_notifications notifications
```

Expected: fails because notification contracts do not exist.

### Task 3: Implement Notifications

**Files:**
- Create: `crates/agent-scheduler/src/notification.rs`
- Create/Modify: `crates/agent-types/src/task.rs`
- Modify: `crates/agent-types/src/event.rs`
- Modify: `crates/agent-types/src/lib.rs`
- Modify: `crates/agent-ui-bridge/src/event.rs`

- [ ] **Step 1: Add scheduler notification contracts**

Implement `TaskNotificationKind`, `TaskNotification`, and `TaskNotificationSink`.

- [ ] **Step 2: Add runtime event payload**

Implement `TaskNotificationRecorded` and add a `RuntimeEventPayload` variant.

- [ ] **Step 3: Add UI normalization**

Map task notification events to UI-safe summaries.

- [ ] **Step 4: Verify GREEN**

```bash
cargo test --workspace m08_task_notifications notifications
```

### Task 4: Complete M08

**Files:**
- Modify: `docs/plans/M08-04-notifications.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [ ] **Step 1: Mark M08 complete**

Set active focus to M09 planning.

- [ ] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m08 scheduler background tasks"
```
