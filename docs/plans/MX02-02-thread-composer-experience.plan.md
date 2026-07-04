# MX02-02 Thread and Composer Experience Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly.

**Goal:** Productize thread navigation, composer controls, and transcript rendering around typed Desktop state.

**Architecture:** `agent-desktop` and `agent-ui-bridge` supply typed thread/composer/transcript data. React reducers derive view state from typed frames and local presentation filters.

**Tech Stack:** Rust serde contracts where needed, React/TypeScript, Vitest, Vite build.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/MX02-02-thread-composer-experience.spec.md`
- Read: `crates/agent-desktop/src/conversation.rs`
- Read: `crates/agent-ui-bridge/src/command.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `apps/desktop/src/components/Composer.tsx`
- Read: `apps/desktop/src/components/ConversationList.tsx`
- Read: `apps/desktop/src/components/TranscriptView.tsx`
- Read: `apps/desktop/src/state/conversation.ts`
- Read: `apps/desktop/src/types.ts`

- [ ] **Step 1: Review current conversation flow**

Expected: identify current active-conversation, start-turn, frame-drain, transcript, task, and diagnostic handling.

### Task 2: Thread Lifecycle UI

**Files:**
- Modify: `apps/desktop/src/components/ConversationList.tsx`
- Modify: `apps/desktop/src/state/conversation.ts`
- Modify: `apps/desktop/src/types.ts`

- [ ] **Step 1: Add thread sections**

Add recent, pinned, and archived sections with search and empty states.

- [ ] **Step 2: Add lifecycle affordances**

Add new/resume/switch/pin/archive/find controls. Use placeholders only where typed commands do not exist yet.

### Task 3: Composer and Transcript

**Files:**
- Modify: `apps/desktop/src/components/Composer.tsx`
- Modify: `apps/desktop/src/components/TranscriptView.tsx`
- Modify: `apps/desktop/src/state/conversation.ts`
- Modify: `apps/desktop/src/types.ts`
- Modify: Rust contracts only if typed data is missing

- [ ] **Step 1: Add composer controls**

Add provider/model selectors, Local/Worktree segmented control, permission mode, context toggle placeholder, slash menu, `$` hint, send/stop/retry states, and goal/progress row.

- [ ] **Step 2: Add typed transcript cards**

Render user, assistant, tool, approval, diff, task, and system events with compact cards and expandable command output.

- [ ] **Step 3: Add redaction assertions**

Ensure secret-like output is not rendered in fixtures, snapshots, or component state.

### Task 4: Verify

**Files:**
- Modify: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`
- Modify: `apps/desktop/src/__tests__/app-shell.test.tsx`

- [ ] **Step 1: Run focused checks**

Run:

```bash
npm --prefix apps/desktop test -- conversation-task-ui app-shell
npm --prefix apps/desktop run build
```

Expected: thread/composer/transcript UI passes focused tests and build.

### Task 5: Commit

- [ ] **Step 1: Commit slice**

Run:

```bash
git diff --check
git add apps/desktop crates/agent-desktop crates/agent-ui-bridge docs/specs/MX02-02-thread-composer-experience.spec.md docs/plans/MX02-02-thread-composer-experience.plan.md
git commit -m "feat: productize desktop thread composer"
```

Expected: commit includes only this slice.
