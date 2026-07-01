# M60 Desktop Daily Driver Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Desktop a credible daily-driver interface for the local coding-agent workflow while keeping runtime behavior in shared Rust crates.

**Architecture:** `agent-ui-bridge` carries typed UI commands and frames, `agent-desktop` carries Desktop view models, Tauri commands expose a narrow typed bridge, and React renders/reduces Desktop state. Runtime logic remains in shared runtime crates.

**Tech Stack:** Rust 2024 edition, serde contracts, Tauri 2 command boundaries, React 18, TypeScript, Vitest, existing UI bridge frames, deterministic fake-provider/eval fixtures, Cargo fmt/clippy/test, `npm --prefix apps/desktop test`, `npm --prefix apps/desktop run build`, and `cargo xtask eval`.

---

### Task 1: Confirm M60 Inputs

**Files:**
- Read: `docs/specs/M60-desktop-daily-driver.spec.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`
- Read: `docs/specs/M21-desktop-gui-mvp.spec.md`
- Read: `docs/plans/M21-desktop-gui-mvp.plan.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-desktop/src/lib.rs`
- Read: `crates/agent-ui-bridge/src/command.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `apps/desktop/src/App.tsx`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src/api.ts`
- Read: `apps/desktop/src/bridge.ts`
- Read: `apps/desktop/src-tauri/src/commands.rs`
- Read: `apps/desktop/src-tauri/src/bridge.rs`

- [x] **Step 1: Review the Desktop baseline**

Baseline reviewed during M60 planning:

- M21 Desktop MVP spec/plan
- current `agent-desktop` view-model modules
- current `agent-ui-bridge` command/frame ownership
- current Tauri command/session bridge modules
- current React shell, component, state, and Vitest coverage
- current `cargo xtask desktop-test` behavior

- [ ] **Step 2: Run baseline checks before implementation**

Run:

```bash
cargo test -p agent-desktop
cargo test -p agent-ui-bridge
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
```

Expected: existing Desktop Rust contracts, bridge contracts, Tauri command tests, frontend tests/build, and default eval suite pass before M60 implementation changes.

### Task 2: M60-01 Workspace Dashboard and Session Readiness

**Files:**
- Create: `docs/specs/M60-01-workspace-dashboard-session.spec.md`
- Create: `docs/plans/M60-01-workspace-dashboard-session.plan.md`
- Modify: `crates/agent-desktop/src/snapshot.rs`
- Modify: `crates/agent-desktop/src/session.rs`
- Modify: `crates/agent-desktop/tests/desktop_daily_driver.rs`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/app.css`
- Modify: `apps/desktop/src/components/*`
- Modify: `apps/desktop/src-tauri/src/workspace_snapshot.rs`
- Modify: `apps/desktop/src-tauri/src/tests/workspace_snapshot.rs`
- Modify: `apps/desktop/src/__tests__/*`

- [x] **Step 1: Plan dashboard/session slice**

Defined typed dashboard state for workspace identity, runtime status, provider readiness, task counts, pending approvals, recent trace/log summaries, and bridge/session diagnostics in M60-01.

- [x] **Step 2: Implement dashboard/session slice**

Built dashboard view models, Tauri dashboard snapshot boundary, Dashboard route, frontend rendering, route defaults, and focused regression tests.

- [x] **Step 3: Verify dashboard/session slice**

Focused Rust, Tauri, frontend, and build checks passed for M60-01.

### Task 3: M60-02 Conversation, Composer, and Transcript Flow

**Files:**
- Create: `docs/specs/M60-02-conversation-composer-transcript.spec.md`
- Create: `docs/plans/M60-02-conversation-composer-transcript.plan.md`
- Modify: `crates/agent-desktop/src/conversation.rs`
- Modify: `crates/agent-ui-bridge/src/command.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/tests/desktop_daily_driver.rs`
- Modify: `apps/desktop/src/state/conversation.ts`
- Modify: `apps/desktop/src/components/Composer.tsx`
- Modify: `apps/desktop/src/components/TranscriptView.tsx`
- Modify: `apps/desktop/src/bridge.ts`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/__tests__/conversation-task-ui.test.tsx`

- [x] **Step 1: Plan conversation/composer slice**

Define how Desktop starts a turn, drains bridge frames, updates transcript state, reports command acknowledgements, and handles recoverable bridge errors.

M60-02 planning is captured in:

- `docs/specs/M60-02-conversation-composer-transcript.spec.md`
- `docs/plans/M60-02-conversation-composer-transcript.plan.md`

- [ ] **Step 2: Implement conversation/composer slice**

Connect composer submission to typed UI commands and render frame-driven transcript/task updates using deterministic fixtures.

- [ ] **Step 3: Verify conversation/composer slice**

Run focused bridge, Tauri, and Vitest tests named in the slice plan.

### Task 4: M60-03 Approval Popup and Visual Diff Review

**Files:**
- Create: `docs/specs/M60-03-approval-diff-review.spec.md`
- Create: `docs/plans/M60-03-approval-diff-review.plan.md`
- Modify: `crates/agent-desktop/src/permission.rs`
- Modify: `crates/agent-desktop/src/diff.rs`
- Modify: `crates/agent-desktop/tests/permissions_diffs.rs`
- Modify: `apps/desktop/src/components/ApprovalDetail.tsx`
- Modify: `apps/desktop/src/components/ApprovalQueue.tsx`
- Modify: `apps/desktop/src/components/DiffPreview.tsx`
- Modify: `apps/desktop/src/state/approvals.ts`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src-tauri/src/approval_commands.rs`
- Modify: `apps/desktop/src-tauri/src/tests/approval_commands.rs`
- Modify: `apps/desktop/src/__tests__/approval-diff-ui.test.tsx`

- [ ] **Step 1: Plan approval/diff slice**

Define explicit decision controls, lease display, request identity, policy rationale, diff file/hunk summaries, redaction, and expired/resolved disabled states.

- [ ] **Step 2: Implement approval/diff slice**

Add Desktop review view models and UI state for approval popups and visual diff review while preserving shared policy and bridge command paths.

- [ ] **Step 3: Verify approval/diff slice**

Run focused Desktop contract, Tauri approval, and frontend approval/diff tests named in the slice plan.

### Task 5: M60-04 Provider Settings, Task Monitor, and Logs

**Files:**
- Create: `docs/specs/M60-04-provider-task-log-surfaces.spec.md`
- Create: `docs/plans/M60-04-provider-task-log-surfaces.plan.md`
- Modify: `crates/agent-desktop/src/settings.rs`
- Modify: `crates/agent-desktop/src/task.rs`
- Modify: `crates/agent-desktop/src/trace.rs`
- Modify: `crates/agent-desktop/tests/settings.rs`
- Modify: `crates/agent-desktop/tests/trace_timeline.rs`
- Modify: `apps/desktop/src/components/SettingsView.tsx`
- Modify: `apps/desktop/src/components/TaskDetail.tsx`
- Modify: `apps/desktop/src/components/TaskList.tsx`
- Modify: `apps/desktop/src/components/TraceTimeline.tsx`
- Modify: `apps/desktop/src/state/settings.ts`
- Modify: `apps/desktop/src/state/tasks.ts`
- Modify: `apps/desktop/src/state/trace.ts`
- Modify: `apps/desktop/src-tauri/src/settings_commands.rs`
- Modify: `apps/desktop/src-tauri/src/trace_commands.rs`
- Modify: `apps/desktop/src-tauri/src/tests/trace_settings_commands.rs`
- Modify: `apps/desktop/src/__tests__/trace-settings-ui.test.tsx`

- [ ] **Step 1: Plan provider/task/log slice**

Define provider/model diagnostics, settings validation, task monitor summaries, bounded runtime log entries, and redacted trace/log rendering.

- [ ] **Step 2: Implement provider/task/log slice**

Polish provider/settings, task monitor, and logs around typed view models and deterministic fixtures.

- [ ] **Step 3: Verify provider/task/log slice**

Run focused settings/task/log Rust tests and Vitest coverage named in the slice plan.

### Task 6: M60-05 Desktop Golden Path Smoke and Closeout

**Files:**
- Create: `docs/specs/M60-05-desktop-golden-path-smoke.spec.md`
- Create: `docs/plans/M60-05-desktop-golden-path-smoke.plan.md`
- Create: `docs/desktop/daily-driver-smoke.md`
- Modify: `crates/agent-desktop/tests/desktop_daily_driver.rs`
- Modify: `crates/agent-ui-bridge/tests/desktop_daily_driver.rs`
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`
- Modify: `xtask/src/lib.rs`
- Modify: `xtask/tests/xtask_plan.rs`
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M60-desktop-daily-driver.plan.md`

- [ ] **Step 1: Plan Desktop smoke slice**

Define the credential-free Desktop bridge simulation or M56-equivalent fixture, including workspace dashboard, composer, transcript, approval, diff, provider settings, task monitor, logs, and trace evidence.

- [ ] **Step 2: Implement Desktop smoke slice**

Add deterministic Rust and frontend smoke coverage, document the daily-driver Desktop path, and extend `cargo xtask desktop-test` if needed so it represents the Desktop product gate.

- [ ] **Step 3: Run final M60 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
```

Expected: all workspace, Desktop, frontend, and eval gates pass.

- [ ] **Step 4: Commit M60 closeout**

Run exact-path staging and commit the final docs/status updates.

### Completion Checklist

- [x] Workspace dashboard/session readiness is visible and tested.
- [ ] Conversation, composer, and transcript flow through shared bridge commands/frames.
- [ ] Approval popup and visual diff review are explicit, bounded, redacted, and policy-backed.
- [ ] Provider/settings diagnostics expose useful status without secrets.
- [ ] Task monitor and logs show bounded runtime evidence.
- [ ] Desktop smoke fixture proves the M56 golden-path shape or a UI-safe bridge simulation.
- [ ] Desktop frontend tests and build are part of the milestone verification path.
- [ ] Final workspace fmt/clippy/tests, frontend checks, and `cargo xtask eval` pass.
