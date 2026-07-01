# M60-03 Approval Popup and Visual Diff Review Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit Desktop approval popup and visual diff review flows with typed decisions, lease/expiry display, disabled terminal states, bounded redacted previews, and focused verification.

**Architecture:** `agent-ui-bridge` carries approval request/resolution frames and `ResolveApproval` commands; `agent-desktop` owns Desktop-safe approval/diff review models and pure transitions; Tauri validates and forwards typed approval commands; React renders review controls and diff previews from typed state. Runtime policy, sandbox, leases, and audit remain in shared runtime crates.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `agent-desktop`, `agent-ui-bridge`, and Tauri approval modules, React 18, TypeScript, Vitest, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M60-03-approval-diff-review.spec.md`
- Read: `docs/specs/M60-desktop-daily-driver.spec.md`
- Read: `docs/plans/M60-desktop-daily-driver.plan.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-desktop/src/permission.rs`
- Read: `crates/agent-desktop/src/diff.rs`
- Read: `crates/agent-desktop/tests/permissions_diffs.rs`
- Read: `apps/desktop/src/state/approvals.ts`
- Read: `apps/desktop/src/components/ApprovalQueue.tsx`
- Read: `apps/desktop/src/components/ApprovalDetail.tsx`
- Read: `apps/desktop/src/components/DiffPreview.tsx`
- Read: `apps/desktop/src-tauri/src/approval_commands.rs`
- Read: `apps/desktop/src-tauri/src/tests/approval_commands.rs`
- Read: `apps/desktop/src/__tests__/approval-diff-ui.test.tsx`

- [x] **Step 1: Review current approval/diff baseline**

Baseline reviewed before planning:

- `agent-desktop` has basic `PermissionDialog` and `DiffReview` transitions.
- Tauri already converts `DesktopResolveApprovalRequest` into `UiCommand::ResolveApproval`.
- React already queues approval requests, validates deny reason, and renders a basic `DiffPreview`.
- The app does not yet model lease/expiry disabled states, resolved terminal controls, command acknowledgement display, file-level diff review selection, or hunk previews.

### Task 2: Add Failing Approval/Diff Contract Tests

**Files:**
- Modify: `crates/agent-desktop/tests/permissions_diffs.rs`
- Modify: `apps/desktop/src/__tests__/approval-diff-ui.test.tsx`
- Modify: `apps/desktop/src-tauri/src/tests/approval_commands.rs`

- [x] **Step 1: Add `agent-desktop` review tests**

Cover:

- approval review status maps pending/allowed/denied/expired/resolved states
- expired or terminal approvals disable decisions with a typed reason
- preview text is bounded and secret-redacted
- diff file reviews reject unsafe paths and empty summaries
- diff hunk previews bound/redact lines and expose truncation

- [x] **Step 2: Extend Tauri approval command tests**

Cover:

- allow decisions include expected lease duration
- deny decisions reject empty reasons
- invalid request ids are redacted
- connected-session acknowledgements expose pending frame count

- [x] **Step 3: Extend frontend approval/diff tests**

Cover:

- approval queue selection and popup/detail rendering
- disabled allow/deny controls for expired/resolved reviews
- command acknowledgement/error status after allow/deny
- visual diff file summaries and hunk lines
- redaction/truncation badges and no secret expansion

- [x] **Step 4: Run focused tests and verify coverage**

Run:

```bash
cargo test -p agent-desktop --test permissions_diffs
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml approval_commands
npm --prefix apps/desktop test -- approval-diff-ui
```

Expected: focused tests cover the new contracts. Implementation landed with the tests in this slice; final focused verification passed after the contracts were implemented.

### Task 3: Implement `agent-desktop` Approval/Diff Review Models

**Files:**
- Modify: `crates/agent-desktop/src/permission.rs`
- Modify: `crates/agent-desktop/src/diff.rs`
- Modify: `crates/agent-desktop/src/lib.rs`
- Modify: `crates/agent-desktop/tests/permissions_diffs.rs`

- [x] **Step 1: Add approval review contracts**

Implement typed approval review structs/enums for status, action/resource labels, policy rationale, preview, lease/expiry display, decision-enabled state, and disabled reason.

- [x] **Step 2: Add visual diff contracts**

Implement diff file review and hunk preview contracts with bounded/redacted lines, relative path validation, selected file state, and terminal status helpers.

- [x] **Step 3: Run focused Desktop contract tests**

Run:

```bash
cargo test -p agent-desktop --test permissions_diffs
```

Expected: approval/diff contract tests pass.

Passed with nine focused Desktop permission/diff contract tests.

### Task 4: Strengthen Tauri Approval Boundary

**Files:**
- Modify: `apps/desktop/src-tauri/src/approval_commands.rs`
- Modify: `apps/desktop/src-tauri/src/tests/approval_commands.rs`

- [x] **Step 1: Preserve typed command conversion**

Keep approval decisions flowing through `UiCommand::ResolveApproval`, and add any narrow helper needed to expose command acknowledgement/error state safely.

- [x] **Step 2: Validate disabled/error inputs before bridge submission**

Ensure empty reasons, invalid request ids, and secret-like invalid ids remain bounded and redacted before returning Tauri errors.

- [x] **Step 3: Run focused Tauri tests**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml approval_commands
```

Expected: Tauri approval command tests pass.

Passed with five focused Tauri approval command tests.

### Task 5: Connect React Approval Popup and Diff Review

**Files:**
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/state/approvals.ts`
- Modify: `apps/desktop/src/components/ApprovalQueue.tsx`
- Modify: `apps/desktop/src/components/ApprovalDetail.tsx`
- Modify: `apps/desktop/src/components/DiffPreview.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/app.css`
- Modify: `apps/desktop/src/__tests__/approval-diff-ui.test.tsx`

- [x] **Step 1: Add TypeScript review state types**

Mirror approval review, decision state, command acknowledgement, disabled reason, diff file, and diff hunk contracts in `types.ts`.

- [x] **Step 2: Reduce approval frames into review state**

Update approval state helpers so pending, resolved, expired, and duplicated approval frames update typed review state and preserve last decision/ack status.

- [x] **Step 3: Render popup/detail disabled states**

Render approval action/resource/rationale, preview, lease/expiry, disabled reason, allow/deny controls, command acknowledgement, and recoverable errors.

- [x] **Step 4: Render visual diff file/hunk review**

Render file summaries, selected file state, bounded hunk lines, redaction/truncation badges, and stable empty states.

- [x] **Step 5: Run focused frontend tests**

Run:

```bash
npm --prefix apps/desktop test -- approval-diff-ui
```

Expected: approval/diff Vitest coverage passes.

Passed with seven focused approval/diff UI tests and the frontend production build.

### Task 6: Verify Slice and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M60-desktop-daily-driver.plan.md`
- Modify: `docs/plans/M60-03-approval-diff-review.plan.md`

- [x] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-desktop --all-targets -- -D warnings
cargo test -p agent-desktop --test permissions_diffs
cargo test -p agent-ui-bridge approval_bridge
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml approval_commands
npm --prefix apps/desktop test -- approval-diff-ui
npm --prefix apps/desktop run build
```

Expected: all focused M60-03 checks pass.

Verification passed:

- `cargo fmt --all -- --check`
- `cargo clippy -p agent-desktop --all-targets -- -D warnings`
- `cargo test -p agent-desktop --test permissions_diffs`
- `cargo test -p agent-ui-bridge --test approval_bridge`
- `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml approval_commands`
- `npm --prefix apps/desktop test -- approval-diff-ui`
- `npm --prefix apps/desktop run build`

- [x] **Step 2: Run status and exact-path commit**

Run:

```bash
git status --short --branch
git add docs/specs/M60-03-approval-diff-review.spec.md \
  docs/plans/M60-03-approval-diff-review.plan.md \
  docs/plans/M60-desktop-daily-driver.plan.md \
  docs/milestones/README.md \
  docs/roadmap/PRODUCT_ROADMAP.md \
  crates/agent-desktop/src/permission.rs \
  crates/agent-desktop/src/diff.rs \
  crates/agent-desktop/src/lib.rs \
  crates/agent-desktop/tests/permissions_diffs.rs \
  apps/desktop/src-tauri/src/approval_commands.rs \
  apps/desktop/src-tauri/src/tests/approval_commands.rs \
  apps/desktop/src/App.tsx \
  apps/desktop/src/types.ts \
  apps/desktop/src/app.css \
  apps/desktop/src/state/approvals.ts \
  apps/desktop/src/components/ApprovalQueue.tsx \
  apps/desktop/src/components/ApprovalDetail.tsx \
  apps/desktop/src/components/DiffPreview.tsx \
  apps/desktop/src/__tests__/approval-diff-ui.test.tsx
git commit -m "feat: add m60 approval diff review"
```

Expected: commit includes only M60-03 docs, source, tests, and status updates.

### Completion Checklist

- [x] M60-03 spec and plan are written.
- [x] Approval review models are typed, serde-friendly, bounded, and redacted.
- [x] Allow/deny decisions use typed bridge commands and validation.
- [x] Expired/resolved/terminal approvals and diffs disable decisions.
- [x] Visual diff review renders file summaries and bounded hunk previews.
- [x] Recoverable approval command errors render without crashing the shell.
- [x] Focused Rust, Tauri, frontend, and build checks pass.
