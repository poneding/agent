# M21-03 Conversation and Task UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render Desktop conversation, transcript, composer, task list, and task detail views from UI-safe snapshots and frames.

**Architecture:** `ruse-desktop` owns snapshot contracts; Tauri exposes `load_workspace_snapshot`; frontend reducers derive conversation and task UI state from snapshots plus `UiJsonFrame`s.

**Tech Stack:** Rust 2024 edition, Tauri 2 commands, `ruse-desktop`, `ruse-ui-bridge`, TypeScript, React, Vitest, Testing Library, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M21-03-conversation-task-ui.spec.md`
- Read: `crates/ruse-desktop/src/snapshot.rs`
- Read: `crates/ruse-desktop/src/conversation.rs`
- Read: `crates/ruse-desktop/src/task.rs`
- Read: `crates/ruse-ui-bridge/src/event.rs`

- [x] **Step 1: Review view-model contracts**

Run:

```bash
sed -n '1,260p' docs/specs/M21-03-conversation-task-ui.spec.md
```

Expected: conversation/task UI uses snapshots and bridge frames, not scheduler or storage internals.

### Task 2: Write RED Tests

**Files:**
- Create: `apps/desktop/src-tauri/src/tests/workspace_snapshot.rs`
- Create: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`

- [x] **Step 1: Add Rust snapshot command tests**

Cover empty workspace snapshot, populated conversations, active-before-terminal task ordering, invalid snapshot diagnostics, and serialization.

- [x] **Step 2: Add frontend reducer/component tests**

Cover empty state, populated conversation list, transcript updates, composer submit, task notifications, task detail selection, and disconnected submit disabled state.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml workspace_snapshot
npm --prefix apps/desktop test -- --run conversation-task-ui
```

Expected: tests fail because snapshot command and UI components are missing.

### Task 3: Implement Conversation and Task UI

**Files:**
- Create: `apps/desktop/src-tauri/src/workspace_snapshot.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Create: `apps/desktop/src/state/conversation.ts`
- Create: `apps/desktop/src/state/tasks.ts`
- Create: `apps/desktop/src/components/ConversationList.tsx`
- Create: `apps/desktop/src/components/TranscriptView.tsx`
- Create: `apps/desktop/src/components/Composer.tsx`
- Create: `apps/desktop/src/components/TaskList.tsx`
- Create: `apps/desktop/src/components/TaskDetail.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/api.ts`
- Modify: `apps/desktop/src/types.ts`

- [x] **Step 1: Add workspace snapshot command**

Implement `load_workspace_snapshot` with deterministic fixture data when no persisted runtime snapshot exists.

- [x] **Step 2: Add reducers**

Implement conversation and task reducers for user messages, model deltas, final responses, diagnostics, and task notifications.

- [x] **Step 3: Add components**

Render conversation list, transcript, composer, task list, and task detail with empty/loading/disconnected states.

- [x] **Step 4: Wire composer command**

Submit prompts as `UiCommand::StartTurn` through M21-02 bridge command wrappers.

### Task 4: Verify

**Files:**
- Read: `apps/desktop/src-tauri/src/tests/workspace_snapshot.rs`
- Read: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml workspace_snapshot
npm --prefix apps/desktop test -- --run conversation-task-ui
```

Expected: focused tests pass.

- [x] **Step 2: Run regression checks**

Run:

```bash
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask check
```

Expected: Desktop frontend tests, frontend build, and Rust workspace checks pass.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M21-03-conversation-task-ui.plan.md`
- Modify: `docs/plans/M21-desktop-gui-mvp.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Mark M21-03 complete**

Update milestone status and set focus to M21-04.
