# MX02-04 Inspector Panels Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly.

**Goal:** Build the tabbed right inspector for context, changes, approvals, tasks, artifacts, and trace.

**Architecture:** `ruse-desktop` supplies typed view models, Tauri exposes existing commands, and React renders tab state and panel presentation.

**Tech Stack:** Rust serde contracts/tests where needed, React/TypeScript, Vitest, Vite.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/MX02-04-inspector-panels.spec.md`
- Read: `crates/ruse-desktop/src/diff.rs`
- Read: `crates/ruse-desktop/src/permission.rs`
- Read: `crates/ruse-desktop/src/task.rs`
- Read: `crates/ruse-desktop/src/trace.rs`
- Read: `apps/desktop/src/components/ApprovalDetail.tsx`
- Read: `apps/desktop/src/components/DiffPreview.tsx`
- Read: `apps/desktop/src/components/TaskDetail.tsx`
- Read: `apps/desktop/src/components/TraceTimeline.tsx`

- [ ] **Step 1: Review current route panels**

Expected: identify reusable approval, diff, task, and trace components to move into inspector tabs.

### Task 2: Build Inspector Contracts and Components

**Files:**
- Modify: `crates/ruse-desktop/src/diff.rs`
- Modify: `crates/ruse-desktop/src/permission.rs`
- Modify: `crates/ruse-desktop/src/task.rs`
- Modify: `crates/ruse-desktop/src/trace.rs`
- Modify: `apps/desktop/src/components/*`
- Modify: `apps/desktop/src/types.ts`

- [ ] **Step 1: Add inspector models where needed**

Add typed artifacts/context/change summaries only when current models are insufficient.

- [ ] **Step 2: Implement tabs**

Render Context, Changes, Approvals, Tasks, Artifacts, and Trace tabs with empty/loading/error states.

### Task 3: Verify

**Files:**
- Modify: `apps/desktop/src/__tests__/approval-diff-ui.test.tsx`
- Modify: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`
- Modify: focused Rust tests

- [ ] **Step 1: Run focused checks**

Run:

```bash
cargo test -p ruse-desktop --test permissions_diffs --test conversation_tasks --test plan_todo_tasks
npm --prefix apps/desktop test -- approval-diff-ui conversation-task-ui desktop-smoke
npm --prefix apps/desktop run build
```

Expected: inspector tabs render typed, safe, actionable panel content.

### Task 4: Commit

- [ ] **Step 1: Commit slice**

Run:

```bash
git diff --check
git add crates/ruse-desktop apps/desktop docs/specs/MX02-04-inspector-panels.spec.md docs/plans/MX02-04-inspector-panels.plan.md
git commit -m "feat: productize desktop inspector panels"
```

Expected: commit succeeds with inspector-only changes.
