# MX02 Desktop Productization Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Productize Desktop into a polished local coding-agent workspace while preserving the shared Rust runtime, typed bridge contracts, and React-as-presentation boundary.

**Architecture:** `agent-model` owns provider/model config contracts, `agent-ui-bridge` owns UI commands/frames/snapshots, `agent-desktop` owns Desktop view models, Tauri owns a narrow command bridge, and React renders typed state. M62 recovery implementation is out of scope for this lane.

**Tech Stack:** Rust 2024 edition, serde contracts, Tauri 2, React 18, TypeScript, Vite, Vitest, CSS variables/plain CSS initially, optional Radix UI primitives/class-variance-authority/lucide-react after dependency/build review, Cargo fmt/clippy/test, `npm --prefix apps/desktop test`, `npm --prefix apps/desktop run build`, `cargo xtask desktop-test`, and `cargo xtask eval`.

---

### Task 1: Confirm MX02 Inputs

**Files:**
- Read: `docs/specs/MX02-desktop-productization.spec.md`
- Read: `docs/PRD.md`
- Read: `docs/architecture/README.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/storage.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `docs/specs/M60-desktop-daily-driver.spec.md`
- Read: `docs/specs/M61-plan-todo-task-ux.spec.md`
- Read: `docs/specs/M62-reliability-recovery-productization.spec.md`
- Read: `apps/desktop/package.json`
- Read: `apps/desktop/src/App.tsx`
- Read: `apps/desktop/src/app.css`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src/components/`
- Read: `apps/desktop/src/state/`
- Read: `apps/desktop/src-tauri/src/`
- Read: `crates/agent-desktop/src/`
- Read: `crates/agent-ui-bridge/src/`
- Read: `crates/agent-model/src/config.rs`

- [x] **Step 1: Review baseline**

Baseline reviewed during MX02 planning:

- M60 Desktop daily-driver contract and current route-centric Desktop shell
- M61 task/plan/todo Desktop surfaces
- M62 recovery boundary so recovery implementation stays separate
- existing Tauri command bridge, frame drain, settings, approval, diff, trace, task, and config modules
- existing frontend tests: `app-shell`, `conversation-task-ui`, `approval-diff-ui`, `trace-settings-ui`, and `desktop-smoke`

- [ ] **Step 2: Run baseline checks before UI implementation**

Run:

```bash
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo test -p agent-desktop
cargo test -p agent-ui-bridge
```

Expected: existing Desktop contracts and UI pass before productization changes.

### Task 2: MX02-01 Desktop App Shell and Navigation

**Files:**
- Read: `docs/specs/MX02-01-desktop-app-shell-navigation.spec.md`
- Read: `docs/plans/MX02-01-desktop-app-shell-navigation.plan.md`
- Modify: `apps/desktop/package.json`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/app.css`
- Modify: `apps/desktop/src/components/`
- Modify: `apps/desktop/src/state/`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/__tests__/app-shell.test.tsx`

- [ ] **Step 1: Review UI foundation dependency choice**

Evaluate Radix UI primitives, class-variance-authority, and lucide-react for local component primitives. If adopted, record the package/build impact in the slice docs and keep generated-code dependencies out of the required workflow.

- [ ] **Step 2: Implement shell**

Refactor `AppShell` into a modern three-column workspace with left sidebar, center transcript/composer, right inspector tabs, bottom terminal drawer placeholder, command palette skeleton, and keyboard shortcut state.

- [ ] **Step 3: Verify shell**

Run:

```bash
npm --prefix apps/desktop test -- app-shell
npm --prefix apps/desktop run build
```

Expected: shell navigation, toggles, command palette skeleton, inspector tabs, and terminal drawer placeholder render without layout regressions.

### Task 3: MX02-02 Thread and Composer Experience

**Files:**
- Read: `docs/specs/MX02-02-thread-composer-experience.spec.md`
- Read: `docs/plans/MX02-02-thread-composer-experience.plan.md`
- Modify: `crates/agent-desktop/src/conversation.rs`
- Modify: `crates/agent-ui-bridge/src/command.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `apps/desktop/src/components/Composer.tsx`
- Modify: `apps/desktop/src/components/ConversationList.tsx`
- Modify: `apps/desktop/src/components/TranscriptView.tsx`
- Modify: `apps/desktop/src/state/conversation.ts`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`
- Modify: `apps/desktop/src/__tests__/app-shell.test.tsx`

- [ ] **Step 1: Implement thread lifecycle UI**

Add create/resume/switch/pin/archive/search/find-in-thread UI state backed by typed thread snapshots.

- [ ] **Step 2: Implement composer and transcript polish**

Add provider/model selectors, local/worktree mode segmented control, permission indicator, context toggle placeholder, slash command menu, `$` skill hints, send/stop/retry states, goal/progress row, typed transcript event rendering, compact tool cards, expandable command output, and redaction tests.

- [ ] **Step 3: Verify thread/composer**

Run:

```bash
npm --prefix apps/desktop test -- conversation-task-ui app-shell
npm --prefix apps/desktop run build
```

Expected: thread and composer state is keyboard-friendly, typed-frame-driven, and secret-safe.

### Task 4: MX02-03 Provider and Model Configuration

**Files:**
- Read: `docs/specs/MX02-03-provider-model-configuration.spec.md`
- Read: `docs/plans/MX02-03-provider-model-configuration.plan.md`
- Modify: `crates/agent-model/src/config.rs`
- Modify: `crates/agent-desktop/src/settings.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `apps/desktop/src/components/SettingsView.tsx`
- Modify: `apps/desktop/src/state/settings.ts`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src-tauri/src/settings_commands.rs`
- Modify: focused tests named by the slice

- [ ] **Step 1: Extend typed provider/model contracts**

Add provider profiles and model profiles with capabilities, diagnostics, defaults, endpoint/secret refs, and no secret values in bridge/Desktop snapshots.

- [ ] **Step 2: Implement settings and per-thread selection UI**

Add provider list, model table/cards, add/edit provider dialog, base URL field, secret ref/API key status, default model selector, per-thread model override, and connection test diagnostics.

- [ ] **Step 3: Verify provider/model**

Run:

```bash
cargo clippy -p agent-model -p agent-desktop -p agent-ui-bridge --all-targets -- -D warnings
cargo test -p agent-model
cargo test -p agent-desktop --test settings
cargo test -p agent-ui-bridge --test config_snapshot
npm --prefix apps/desktop test -- trace-settings-ui
npm --prefix apps/desktop run build
```

Expected: provider/model snapshots are typed, diagnosable, capability-aware, and secret-safe.

### Task 5: MX02-04 Inspector Panels for Changes, Approvals, Tasks, and Artifacts

**Files:**
- Read: `docs/specs/MX02-04-inspector-panels.spec.md`
- Read: `docs/plans/MX02-04-inspector-panels.plan.md`
- Modify: `crates/agent-desktop/src/diff.rs`
- Modify: `crates/agent-desktop/src/permission.rs`
- Modify: `crates/agent-desktop/src/task.rs`
- Modify: `crates/agent-desktop/src/trace.rs`
- Modify: `apps/desktop/src/components/`
- Modify: `apps/desktop/src/state/`
- Modify: `apps/desktop/src/types.ts`
- Modify: focused tests named by the slice

- [ ] **Step 1: Implement inspector tabs**

Add Context, Changes, Approvals, Tasks, Artifacts, and Trace tabs with empty/loading/redacted states.

- [ ] **Step 2: Implement typed panel content**

Changes show file list, hunk preview, staged/unstaged badges, and placeholders for stage/revert/inline comment. Approvals show approve once/session/deny, risk rationale, sandbox/network scope, expiration, and auto-review placeholder. Tasks show plan progress, active todo, waiting approval, notifications, and artifact links. Artifacts show typed metadata previews. Trace shows a compact runtime timeline.

- [ ] **Step 3: Verify inspector**

Run:

```bash
cargo test -p agent-desktop --test permissions_diffs --test conversation_tasks --test plan_todo_tasks
npm --prefix apps/desktop test -- approval-diff-ui conversation-task-ui desktop-smoke
npm --prefix apps/desktop run build
```

Expected: inspector panels are typed, redacted, actionable, and responsive.

### Task 6: MX02-05 Desktop Product Smoke and Documentation

**Files:**
- Read: `docs/specs/MX02-05-desktop-product-smoke-docs.spec.md`
- Read: `docs/plans/MX02-05-desktop-product-smoke-docs.plan.md`
- Create: `docs/desktop/desktop-productization.md`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`
- Modify: `xtask/` if `cargo xtask desktop-test` needs smoke coverage updates
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Implement product smoke**

Cover open workspace, create thread, choose provider/model, submit prompt, receive typed frames, show approval, show diff, show task progress, open settings, and verify no secret leakage.

- [ ] **Step 2: Document workflow**

Document the productized Desktop workspace and the smoke command in `docs/desktop/desktop-productization.md`.

- [ ] **Step 3: Run final gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask desktop-test
cargo xtask eval
```

Expected: Desktop Productization is implemented, documented, secret-safe, and does not regress existing runtime/product gates.

### Task 7: Commit Discipline

**Files:**
- Stage only Desktop Productization docs/source for each slice.
- Do not stage unrelated M62 recovery files unless explicitly working in the M62 recovery lane.

- [x] **Step 1: Commit milestone planning**

After MX02 milestone and slice docs are created, run:

```bash
git diff --check
git add README.md docs/roadmap/PRODUCT_ROADMAP.md docs/milestones/README.md docs/specs/MX02*.md docs/plans/MX02*.md
git commit -m "docs: add desktop productization plan"
```

Expected: milestone planning commits independently from M62 recovery implementation.
