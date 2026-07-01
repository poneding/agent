# M60-04 Provider Settings, Task Monitor, and Logs Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Desktop provider/settings diagnostics, task monitor summaries, and bounded runtime logs over typed shared bridge/Desktop contracts.

**Architecture:** `agent-ui-bridge` carries UI-safe config/task/trace contracts; `agent-desktop` owns provider/settings, task monitor, and log view models; Tauri exposes narrow read-only settings and trace/log commands; React renders typed snapshots and local reducers. Runtime provider selection, scheduler semantics, trace generation, policy, and audit remain in shared runtime crates.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `agent-desktop`, `agent-ui-bridge`, and Tauri settings/trace modules, React 18, TypeScript, Vitest, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M60-04-provider-task-log-surfaces.spec.md`
- Read: `docs/specs/M60-desktop-daily-driver.spec.md`
- Read: `docs/plans/M60-desktop-daily-driver.plan.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-desktop/src/settings.rs`
- Read: `crates/agent-desktop/src/task.rs`
- Read: `crates/agent-desktop/src/trace.rs`
- Read: `crates/agent-desktop/tests/settings.rs`
- Read: `crates/agent-desktop/tests/trace_timeline.rs`
- Read: `apps/desktop/src/components/SettingsView.tsx`
- Read: `apps/desktop/src/components/TaskList.tsx`
- Read: `apps/desktop/src/components/TaskDetail.tsx`
- Read: `apps/desktop/src/components/TraceTimeline.tsx`
- Read: `apps/desktop/src/state/settings.ts`
- Read: `apps/desktop/src/state/tasks.ts`
- Read: `apps/desktop/src/state/trace.ts`
- Read: `apps/desktop/src-tauri/src/settings_commands.rs`
- Read: `apps/desktop/src-tauri/src/trace_commands.rs`
- Read: `apps/desktop/src-tauri/src/tests/trace_settings_commands.rs`
- Read: `apps/desktop/src/__tests__/trace-settings-ui.test.tsx`

- [x] **Step 1: Review current provider/task/log baseline**

Baseline reviewed before planning:

- `agent-desktop` has basic settings validation, task items/reviews, and trace timelines.
- Tauri has read-only config snapshot, settings validation, and trace loading commands.
- React has settings, task list/detail, and trace timeline components.
- Missing M60-04 pieces are provider capability summaries, task monitor aggregation/errors, runtime log entry contracts, richer task detail rendering, and explicit log rendering tests.

### Task 2: Add Failing Provider/Task/Log Contract Tests

**Files:**
- Modify: `crates/agent-desktop/tests/settings.rs`
- Modify: `crates/agent-desktop/tests/trace_timeline.rs`
- Modify: `crates/agent-desktop/tests/conversation_tasks.rs`
- Modify: `apps/desktop/src-tauri/src/tests/trace_settings_commands.rs`
- Modify: `apps/desktop/src/__tests__/trace-settings-ui.test.tsx`

- [x] **Step 1: Add provider/settings tests**

Cover:

- provider settings summary exposes selected provider/model and capabilities
- provider diagnostics are bounded/redacted
- invalid settings validation returns typed diagnostics without mutation

- [x] **Step 2: Add task monitor tests**

Cover:

- task monitor counts active, waiting approval, failed, and terminal states from typed `TaskState`
- selected task detail includes latest run summary and artifact count
- task errors are bounded/redacted and recoverable/fatal state is preserved

- [x] **Step 3: Add runtime log tests**

Cover:

- runtime log entries derive severity/source/message from trace evidence
- logs sort by sequence and reject duplicates where required
- log messages are bounded/redacted and mark truncation

- [x] **Step 4: Extend frontend/Tauri tests**

Cover:

- settings provider capability rendering
- task monitor counts/detail rendering
- runtime log rendering and empty states
- no secret exposure in diagnostics/logs

- [x] **Step 5: Run focused tests and verify coverage**

Run:

```bash
cargo test -p agent-desktop --test settings
cargo test -p agent-desktop --test trace_timeline
cargo test -p agent-desktop --test conversation_tasks
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml trace_settings_commands
npm --prefix apps/desktop test -- trace-settings-ui
```

Expected: focused tests cover the new contracts. Implementation landed with the tests in this slice; final focused verification passed after the contracts were implemented.

### Task 3: Implement `agent-desktop` Provider/Task/Log Models

**Files:**
- Modify: `crates/agent-desktop/src/settings.rs`
- Modify: `crates/agent-desktop/src/task.rs`
- Modify: `crates/agent-desktop/src/trace.rs`
- Modify: `crates/agent-desktop/src/lib.rs`
- Modify: `crates/agent-desktop/tests/settings.rs`
- Modify: `crates/agent-desktop/tests/trace_timeline.rs`
- Modify: `crates/agent-desktop/tests/conversation_tasks.rs`

- [x] **Step 1: Add provider settings summary contracts**

Implement provider summaries, selected provider/model validation state, diagnostic summaries, and validation result contracts derived from `DesktopConfigSnapshot` and `DesktopSettings`.

- [x] **Step 2: Add task monitor contracts**

Implement task monitor counts, task detail summaries, artifact counts, latest run summaries, and task monitor errors from typed task items/reviews.

- [x] **Step 3: Add runtime log contracts**

Implement bounded runtime log entries and builders from trace timeline items or trace entries, including severity/source/message/truncation and redaction.

- [x] **Step 4: Run focused Desktop contract tests**

Run:

```bash
cargo test -p agent-desktop --test settings
cargo test -p agent-desktop --test trace_timeline
cargo test -p agent-desktop --test conversation_tasks
```

Expected: Desktop provider/task/log contract tests pass.

Passed with focused settings, trace timeline, and conversation task tests.

### Task 4: Strengthen Tauri Settings and Trace Boundaries

**Files:**
- Modify: `apps/desktop/src-tauri/src/settings_commands.rs`
- Modify: `apps/desktop/src-tauri/src/trace_commands.rs`
- Modify: `apps/desktop/src-tauri/src/tests/trace_settings_commands.rs`

- [x] **Step 1: Expose typed provider/settings summaries**

Adapt existing config snapshot helpers to return redacted provider/settings summary state where needed.

- [x] **Step 2: Expose bounded log evidence**

Adapt trace loading helpers so runtime log entries are derived from bounded/redacted trace evidence.

- [x] **Step 3: Run focused Tauri tests**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml trace_settings_commands
```

Expected: Tauri settings/trace command tests pass.

Passed with seven focused Tauri trace/settings command tests.

### Task 5: Connect React Provider Settings, Task Monitor, and Logs

**Files:**
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/state/settings.ts`
- Modify: `apps/desktop/src/state/tasks.ts`
- Modify: `apps/desktop/src/state/trace.ts`
- Modify: `apps/desktop/src/components/SettingsView.tsx`
- Modify: `apps/desktop/src/components/TaskList.tsx`
- Modify: `apps/desktop/src/components/TaskDetail.tsx`
- Modify: `apps/desktop/src/components/TraceTimeline.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/app.css`
- Modify: `apps/desktop/src/__tests__/trace-settings-ui.test.tsx`

- [x] **Step 1: Add TypeScript provider/task/log types**

Mirror provider settings summary, task monitor, task detail, task monitor error, and runtime log contracts in `types.ts`.

- [x] **Step 2: Render provider/settings diagnostics**

Update settings rendering to show provider capabilities, selected provider/model readiness, validation diagnostics, and redacted config diagnostics.

- [x] **Step 3: Render task monitor summaries**

Update task list/detail and state helpers to show counts, latest run summaries, artifact counts, task errors, and stable empty states.

- [x] **Step 4: Render runtime logs**

Update trace/log components to show bounded runtime log entries with severity/source labels and redaction/truncation flags.

- [x] **Step 5: Run focused frontend tests**

Run:

```bash
npm --prefix apps/desktop test -- trace-settings-ui
```

Expected: provider/task/log Vitest coverage passes.

Passed with six focused trace/settings UI tests and the frontend production build.

### Task 6: Verify Slice and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M60-desktop-daily-driver.plan.md`
- Modify: `docs/plans/M60-04-provider-task-log-surfaces.plan.md`

- [x] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-desktop --all-targets -- -D warnings
cargo test -p agent-desktop --test settings
cargo test -p agent-desktop --test trace_timeline
cargo test -p agent-desktop --test conversation_tasks
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml trace_settings_commands
npm --prefix apps/desktop test -- trace-settings-ui
npm --prefix apps/desktop run build
```

Expected: all focused M60-04 checks pass.

Verification passed:

- `cargo fmt --all -- --check`
- `cargo clippy -p agent-desktop --all-targets -- -D warnings`
- `cargo test -p agent-desktop --test settings`
- `cargo test -p agent-desktop --test trace_timeline`
- `cargo test -p agent-desktop --test conversation_tasks`
- `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml trace_settings_commands`
- `npm --prefix apps/desktop test -- trace-settings-ui`
- `npm --prefix apps/desktop run build`

- [x] **Step 2: Run status and exact-path commit**

Run:

```bash
git status --short --branch
git add docs/specs/M60-04-provider-task-log-surfaces.spec.md \
  docs/plans/M60-04-provider-task-log-surfaces.plan.md \
  docs/plans/M60-desktop-daily-driver.plan.md \
  docs/milestones/README.md \
  docs/roadmap/PRODUCT_ROADMAP.md \
  crates/agent-desktop/src/settings.rs \
  crates/agent-desktop/src/task.rs \
  crates/agent-desktop/src/trace.rs \
  crates/agent-desktop/src/lib.rs \
  crates/agent-desktop/tests/settings.rs \
  crates/agent-desktop/tests/trace_timeline.rs \
  crates/agent-desktop/tests/conversation_tasks.rs \
  apps/desktop/src-tauri/src/settings_commands.rs \
  apps/desktop/src-tauri/src/trace_commands.rs \
  apps/desktop/src-tauri/src/tests/trace_settings_commands.rs \
  apps/desktop/src/App.tsx \
  apps/desktop/src/types.ts \
  apps/desktop/src/app.css \
  apps/desktop/src/state/settings.ts \
  apps/desktop/src/state/tasks.ts \
  apps/desktop/src/state/trace.ts \
  apps/desktop/src/components/SettingsView.tsx \
  apps/desktop/src/components/TaskList.tsx \
  apps/desktop/src/components/TaskDetail.tsx \
  apps/desktop/src/components/TraceTimeline.tsx \
  apps/desktop/src/__tests__/trace-settings-ui.test.tsx
git commit -m "feat: add m60 provider task log surfaces"
```

Expected: commit includes only M60-04 docs, source, tests, and status updates.

### Completion Checklist

- [x] M60-04 spec and plan are written.
- [x] Provider/settings summaries are typed, serde-friendly, bounded, and redacted.
- [x] Settings validation remains read-only and returns typed diagnostics.
- [x] Task monitor state exposes counts, detail, latest run summaries, artifacts, and errors.
- [x] Runtime log entries are bounded, redacted, sorted, and severity-labeled.
- [x] React renders provider diagnostics, task monitor, and logs from typed state.
- [x] Focused Rust, Tauri, frontend, and build checks pass.
