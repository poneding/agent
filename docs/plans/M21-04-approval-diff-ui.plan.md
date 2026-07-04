# M21-04 Approval and Diff UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Desktop approval queue, approval details, diff preview, and typed approval resolution commands.

**Architecture:** Approval frames come from `agent-ui-bridge`; frontend stores UI-only approval state; decisions return through `UiCommand::ResolveApproval` via the M21 bridge session.

**Tech Stack:** Rust 2024 edition, Tauri 2 commands, `agent-ui-bridge`, `agent-desktop`, TypeScript, React, Vitest, Testing Library, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M21-04-approval-diff-ui.spec.md`
- Read: `crates/agent-ui-bridge/src/event.rs`
- Read: `crates/agent-ui-bridge/src/command.rs`
- Read: `crates/agent-desktop/src/permission.rs`
- Read: `crates/agent-desktop/src/diff.rs`

- [x] **Step 1: Review approval boundaries**

Run:

```bash
sed -n '1,260p' docs/specs/M21-04-approval-diff-ui.spec.md
```

Expected: Desktop emits approval decisions and never creates approval leases.

### Task 2: Write RED Tests

**Files:**
- Create: `apps/desktop/src-tauri/src/tests/approval_commands.rs`
- Create: `apps/desktop/src/__tests__/approval-diff-ui.test.tsx`

- [x] **Step 1: Add Rust approval command tests**

Cover allow, deny, empty deny reason, invalid request id, bridge command conversion, and redacted diagnostics.

- [x] **Step 2: Add frontend approval/diff tests**

Cover queued approvals, selected approval details, resolved removal, expired removal, redacted preview, truncated diff, allow action, and deny validation.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml approval_commands
npm --prefix apps/desktop test -- --run approval-diff-ui
```

Expected: tests fail because approval command and UI modules are missing.

### Task 3: Implement Approval and Diff UI

**Files:**
- Create: `apps/desktop/src-tauri/src/approval_commands.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Create: `apps/desktop/src/state/approvals.ts`
- Create: `apps/desktop/src/components/ApprovalQueue.tsx`
- Create: `apps/desktop/src/components/ApprovalDetail.tsx`
- Create: `apps/desktop/src/components/DiffPreview.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/api.ts`
- Modify: `apps/desktop/src/types.ts`

- [x] **Step 1: Add resolve command**

Implement `DesktopResolveApprovalRequest` and `resolve_approval` command conversion to `UiCommand::ResolveApproval`.

- [x] **Step 2: Add approval reducer**

Implement add/update/remove behavior for approval requested, resolved, expired, cancelled, and consumed frames.

- [x] **Step 3: Add UI components**

Render pending queue, details, allow/deny controls, diff preview summary, truncation, and redaction indicators.

- [x] **Step 4: Wire bridge submission**

Send allow/deny through M21-02 `sendUiCommand` wrapper and render command diagnostics.

### Task 4: Verify

**Files:**
- Read: `apps/desktop/src-tauri/src/tests/approval_commands.rs`
- Read: `apps/desktop/src/__tests__/approval-diff-ui.test.tsx`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml approval_commands
npm --prefix apps/desktop test -- --run approval-diff-ui
```

Expected: approval and diff tests pass.

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
- Modify: `docs/plans/M21-04-approval-diff-ui.plan.md`
- Modify: `docs/plans/M21-desktop-gui-mvp.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Mark M21-04 complete**

Update milestone status and set focus to M21-05.
