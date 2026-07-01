# M60-05 Desktop Golden Path Smoke and Closeout Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove the M60 Desktop daily-driver workflow with a deterministic, credential-free smoke fixture and close out the milestone.

**Architecture:** Existing `agent-ui-bridge` command/frame contracts feed existing `agent-desktop` view models; Tauri commands and React render typed snapshots. `xtask` orchestrates the Desktop product gate. Runtime behavior remains in shared runtime crates, not Desktop presentation code.

**Tech Stack:** Rust 2024 edition, serde contracts, existing Desktop and UI bridge test fixtures, Tauri 2 command tests, React 18, TypeScript, Vitest, npm build, Cargo fmt/clippy/test, `cargo xtask desktop-test`, and `cargo xtask eval`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M60-05-desktop-golden-path-smoke.spec.md`
- Read: `docs/specs/M60-desktop-daily-driver.spec.md`
- Read: `docs/plans/M60-desktop-daily-driver.plan.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-desktop/src/lib.rs`
- Read: `crates/agent-desktop/tests/*`
- Read: `crates/agent-ui-bridge/tests/desktop_daily_driver.rs`
- Read: `apps/desktop/src/App.tsx`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`
- Read: `xtask/src/lib.rs`
- Read: `xtask/tests/xtask_plan.rs`

- [x] **Step 1: Review current Desktop smoke baseline**

Current baseline before implementation:

- M60-01 through M60-04 are complete.
- `apps/desktop/src/__tests__/desktop-smoke.test.tsx` navigates the Desktop routes but does not yet assert a full M56-shaped workflow.
- `crates/agent-ui-bridge/tests/desktop_daily_driver.rs` covers start-turn serialization and basic frame encoding.
- `crates/agent-desktop/tests/desktop_daily_driver.rs` does not exist yet and should be created for the full Desktop contract fixture.
- `cargo xtask desktop-test` currently runs only `cargo test -p agent-desktop`.

### Task 2: Add Desktop Daily-Driver Contract Smoke

**Files:**
- Create: `crates/agent-desktop/tests/desktop_daily_driver.rs`

- [ ] **Step 1: Build the Rust fixture**

Create stable workspace, session, dashboard, conversation, approval, diff, provider/settings, task monitor, and trace/log fixture values using existing typed contracts.

- [ ] **Step 2: Assert M56-shaped evidence**

Verify the fixture contains:

- connected workspace/session readiness
- active conversation and completed submission state
- transcript evidence for prompt, progress, and final response
- pending approval with lease and redacted preview
- visual diff review with bounded hunks
- provider/model readiness and capabilities
- task monitor counts for active, waiting approval, failed, and terminal tasks
- trace timeline and runtime logs for edit, command/test, permission, and final evidence

- [ ] **Step 3: Assert redaction and serialization**

Serialize the complete fixture and verify secret-like tokens, raw API keys, passwords, and hidden tool input names do not leak.

### Task 3: Strengthen UI Bridge Smoke Frames

**Files:**
- Modify: `crates/agent-ui-bridge/tests/desktop_daily_driver.rs`

- [ ] **Step 1: Add approval resolution command coverage**

Verify Desktop approval resolution commands serialize and validate through typed UI bridge contracts.

- [ ] **Step 2: Add M56-shaped frame sequence**

Build a deterministic frame sequence for user message, model delta, task notification, approval requested, approval resolved, final response, and done frames. Verify JSON frames remain one-line and parseable.

- [ ] **Step 3: Assert bridge redaction boundaries**

Ensure fixture frames do not expose secret-like values or unbounded raw payloads.

### Task 4: Strengthen Frontend Smoke Fixture

**Files:**
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`

- [ ] **Step 1: Expand the AppShell fixture**

Add realistic dashboard counts, conversation state, pending approval, task states, provider capabilities, settings diagnostics, trace items, and runtime logs.

- [ ] **Step 2: Assert each Desktop route shows golden-path evidence**

Navigate dashboard, conversation, tasks, approvals, trace, and settings. Assert the UI exposes the expected evidence and does not render secret-like values.

- [ ] **Step 3: Assert composer and approval controls**

Verify prompt submission and approval controls call their typed handlers with stable values.

### Task 5: Make `cargo xtask desktop-test` the Product Gate

**Files:**
- Modify: `xtask/src/lib.rs`
- Modify: `xtask/tests/xtask_plan.rs`
- Modify: `README.md`

- [ ] **Step 1: Update command planning**

Expand `Task::DesktopTest` to run:

```bash
cargo test -p agent-desktop
cargo test -p agent-ui-bridge --test desktop_daily_driver
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
```

- [ ] **Step 2: Update parser/help/readme expectations**

Update `xtask` tests and README wording so `desktop-test` is documented as the Desktop product gate.

### Task 6: Add Desktop Smoke Documentation

**Files:**
- Create: `docs/desktop/daily-driver-smoke.md`
- Modify: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M60-desktop-daily-driver.plan.md`
- Modify: `docs/plans/M60-05-desktop-golden-path-smoke.plan.md`

- [ ] **Step 1: Document the daily-driver smoke path**

Describe what `cargo xtask desktop-test` runs, what evidence the fixture covers, where live provider coverage remains out of scope, and how the smoke path maps to M56.

- [ ] **Step 2: Update roadmap and milestone status**

Mark GP-15/GP-16 and M60 status according to implementation results. Update the M60 slice index and active work handoff.

### Task 7: Run Final M60 Gates and Commit

**Files:**
- Modify: `docs/plans/M60-05-desktop-golden-path-smoke.plan.md`

- [ ] **Step 1: Run focused Desktop smoke checks**

Run:

```bash
cargo fmt --all -- --check
cargo test -p agent-desktop --test desktop_daily_driver
cargo test -p agent-ui-bridge --test desktop_daily_driver
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
npm --prefix apps/desktop test -- desktop-smoke
npm --prefix apps/desktop run build
cargo test -p xtask --test xtask_plan
cargo xtask desktop-test
```

Expected: all focused Desktop smoke checks pass.

- [ ] **Step 2: Run final M60 gates**

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

- [ ] **Step 3: Commit closeout**

Run exact-path staging for M60-05 files and commit:

```bash
git commit -m "feat: add m60 desktop golden path smoke"
```

### Completion Checklist

- [x] M60-05 spec and plan are written.
- [ ] `agent-desktop` has a full daily-driver contract smoke fixture.
- [ ] `agent-ui-bridge` has M56-shaped Desktop command/frame smoke coverage.
- [ ] Frontend Desktop smoke asserts dashboard, conversation, tasks, approvals, trace, settings, and redaction evidence.
- [ ] `cargo xtask desktop-test` runs the Desktop product gate.
- [ ] Desktop daily-driver smoke docs are written.
- [ ] Final M60 workspace, frontend, and eval gates pass.
- [ ] M60 roadmap, milestone, and plan statuses are closed out.
