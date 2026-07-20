# M60-02 Conversation, Composer, and Transcript Flow Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the Desktop conversation composer to typed bridge turn submission, bounded frame draining, transcript updates, command acknowledgements, and recoverable bridge errors.

**Architecture:** `ruse-ui-bridge` owns command/frame contracts and validation; `ruse-desktop` owns Desktop-safe conversation view models and pure helpers; Tauri exposes narrow typed command/frame boundaries; React renders and reduces typed state. Runtime side effects remain in shared runtime, policy, sandbox, audit, and bridge paths.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `ruse-desktop`, `ruse-ui-bridge`, and Tauri bridge modules, React 18, TypeScript, Vitest, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M60-02-conversation-composer-transcript.spec.md`
- Read: `docs/specs/M60-desktop-daily-driver.spec.md`
- Read: `docs/plans/M60-desktop-daily-driver.plan.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/ruse-desktop/src/conversation.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `apps/desktop/src-tauri/src/bridge.rs`
- Read: `apps/desktop/src-tauri/src/tests/bridge_session.rs`
- Read: `apps/desktop/src/state/conversation.ts`
- Read: `apps/desktop/src/components/Composer.tsx`
- Read: `apps/desktop/src/components/TranscriptView.tsx`
- Read: `apps/desktop/src/App.tsx`
- Read: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`

- [x] **Step 1: Review current conversation/composer baseline**

Baseline reviewed before planning:

- `UiCommand::StartTurn` already exists and validates non-empty prompts.
- `DesktopRuntimeSession` already validates commands, acknowledges accepted commands, and drains bounded `UiJsonFrame` queues.
- React already builds a typed start-turn envelope and has a minimal transcript/task reducer.
- The app does not yet drain frames into state, expose acknowledgements, or model submission/error state with Desktop-safe contracts.

### Task 2: Add Failing Conversation Contract Tests

**Files:**
- Create: `crates/ruse-desktop/tests/desktop_conversation.rs`
- Create: `crates/ruse-ui-bridge/tests/desktop_daily_driver.rs`
- Modify: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`

- [x] **Step 1: Add `ruse-desktop` conversation view-model tests**

Cover:

- prompt previews are bounded and secret-redacted
- transcript entries serialize with snake_case roles
- command acknowledgements update current submission state
- done frames mark the current submission complete
- error frames become bounded recoverable bridge notices

- [x] **Step 2: Add bridge command/frame tests**

Cover:

- `StartTurn` command serializes and validates for Desktop clients
- empty prompt is rejected before acknowledgement
- done and recoverable error frames serialize as single-line JSON

- [x] **Step 3: Extend Vitest coverage**

Cover:

- composer submission adds a pending submission and calls `sendUiCommand`
- accepted acknowledgement is displayed
- drained frames update transcript/task state
- bridge errors render as recoverable diagnostics

- [x] **Step 4: Run focused tests and verify coverage**

Run:

```bash
cargo test -p ruse-desktop --test desktop_conversation
cargo test -p ruse-ui-bridge desktop_daily_driver
npm --prefix apps/desktop test -- conversation-task-ui
```

Expected: focused tests cover the new contracts. Implementation landed with the tests in this slice; final focused verification passed after the contracts were implemented.

### Task 3: Implement `ruse-desktop` Conversation Model

**Files:**
- Modify: `crates/ruse-desktop/src/conversation.rs`
- Modify: `crates/ruse-desktop/src/lib.rs`
- Modify: `crates/ruse-desktop/tests/desktop_conversation.rs`

- [x] **Step 1: Add conversation contracts**

Implement typed structs/enums for transcript entries, transcript roles, turn submission state, command acknowledgements, bridge error notices, and conversation diagnostics.

- [x] **Step 2: Add pure helpers**

Implement bounded/redacted prompt preview creation and helper methods for applying accepted acknowledgements, done frames, and error frames to Desktop conversation state.

- [x] **Step 3: Run focused Desktop contract tests**

Run:

```bash
cargo test -p ruse-desktop --test desktop_conversation
```

Expected: Desktop conversation contract tests pass.

Passed with four focused Desktop conversation contract tests.

### Task 4: Strengthen Bridge and Tauri Boundaries

**Files:**
- Modify: `crates/ruse-ui-bridge/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/tests/desktop_daily_driver.rs`
- Modify: `apps/desktop/src-tauri/src/bridge.rs`
- Modify: `apps/desktop/src-tauri/src/tests/bridge_session.rs`

- [x] **Step 1: Add bridge command/frame coverage**

Keep `StartTurn` validation in `ruse-ui-bridge` and add focused Desktop daily-driver tests for command serialization, validation, done frames, and recoverable error frames.

- [x] **Step 2: Add deterministic Tauri frame fixtures if needed**

If the frontend needs testable frame drain behavior beyond existing helpers, add narrow test-only frame enqueue helpers that preserve bounded queue and redaction behavior.

No new Tauri frame fixture helper was required; existing bridge-session helpers covered acknowledgement, frame drain, disconnected, and redaction behavior.

- [x] **Step 3: Run focused bridge/Tauri tests**

Run:

```bash
cargo test -p ruse-ui-bridge desktop_daily_driver
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bridge_session
```

Expected: bridge and Tauri command/frame tests pass.

Passed with the `ruse-ui-bridge` Desktop daily-driver integration test and Tauri bridge-session test.

### Task 5: Connect React Composer, Transcript, and Frame Drain

**Files:**
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/bridge.ts`
- Modify: `apps/desktop/src/state/conversation.ts`
- Modify: `apps/desktop/src/components/Composer.tsx`
- Modify: `apps/desktop/src/components/TranscriptView.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/app.css`
- Modify: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`

- [x] **Step 1: Add TypeScript conversation state types**

Mirror the Rust/Tauri conversation response shape in `types.ts`, including submission status, command ack, pending frame count, and recoverable error notices.

- [x] **Step 2: Apply command acknowledgement state**

Update composer submission flow so accepted acknowledgements update conversation state and rejected/disconnected errors become recoverable diagnostics.

- [x] **Step 3: Drain frames into conversation/task state**

Call `drainUiFrames` after accepted submission and on a bounded polling interval while connected. Reduce event, done, and error frames into transcript, task, submission, and diagnostic state.

- [x] **Step 4: Render acknowledgement and recoverable errors**

Display current submission status, pending frame count, recoverable bridge errors, and transcript entries without exposing raw secrets or hidden runtime data.

- [x] **Step 5: Run focused frontend tests**

Run:

```bash
npm --prefix apps/desktop test -- conversation-task-ui
```

Expected: conversation/composer/transcript Vitest coverage passes.

Passed with six focused conversation-task UI tests and the frontend production build.

### Task 6: Verify Slice and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M60-desktop-daily-driver.plan.md`
- Modify: `docs/plans/M60-02-conversation-composer-transcript.plan.md`

- [x] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-desktop --all-targets -- -D warnings
cargo test -p ruse-desktop --test desktop_conversation
cargo test -p ruse-ui-bridge desktop_daily_driver
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bridge_session
npm --prefix apps/desktop test -- conversation-task-ui
npm --prefix apps/desktop run build
```

Expected: all focused M60-02 checks pass.

Verification passed:

- `cargo fmt --all -- --check`
- `cargo clippy -p ruse-desktop --all-targets -- -D warnings`
- `cargo test -p ruse-desktop --test desktop_conversation`
- `cargo test -p ruse-ui-bridge --test desktop_daily_driver`
- `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bridge_session`
- `npm --prefix apps/desktop test -- conversation-task-ui`
- `npm --prefix apps/desktop run build`

- [x] **Step 2: Run status and exact-path commit**

Run:

```bash
git status --short --branch
git add docs/specs/M60-02-conversation-composer-transcript.spec.md \
  docs/plans/M60-02-conversation-composer-transcript.plan.md \
  docs/plans/M60-desktop-daily-driver.plan.md \
  docs/milestones/README.md \
  crates/ruse-desktop/src/conversation.rs \
  crates/ruse-desktop/src/lib.rs \
  crates/ruse-desktop/tests/desktop_conversation.rs \
  crates/ruse-ui-bridge/src/command.rs \
  crates/ruse-ui-bridge/src/frame.rs \
  crates/ruse-ui-bridge/tests/desktop_daily_driver.rs \
  apps/desktop/src-tauri/src/bridge.rs \
  apps/desktop/src-tauri/src/tests/bridge_session.rs \
  apps/desktop/src/App.tsx \
  apps/desktop/src/bridge.ts \
  apps/desktop/src/types.ts \
  apps/desktop/src/app.css \
  apps/desktop/src/state/conversation.ts \
  apps/desktop/src/components/Composer.tsx \
  apps/desktop/src/components/TranscriptView.tsx \
  apps/desktop/src/__tests__/conversation-task-ui.test.tsx
git commit -m "feat: add m60 conversation composer flow"
```

Expected: commit includes only M60-02 docs, source, tests, and status updates.

### Completion Checklist

- [x] M60-02 spec and plan are written.
- [x] Conversation view models are typed, serde-friendly, bounded, and redacted.
- [x] Start-turn submission uses typed bridge commands and validation.
- [x] Command acknowledgements are visible in Desktop state.
- [x] Bounded frame drains update transcript and task state.
- [x] Recoverable bridge errors render without crashing the shell.
- [x] Focused Rust, Tauri, frontend, and build checks pass.
