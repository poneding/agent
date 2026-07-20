# M60-01 Workspace Dashboard and Session Readiness Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a typed Desktop dashboard/readiness slice that summarizes workspace, session, provider, task, approval, and diagnostic state without moving runtime behavior into Desktop presentation code.

**Architecture:** `ruse-desktop` owns dashboard view models and pure aggregation helpers; Tauri adapts current bridge/session state into typed commands; React renders a dashboard route from typed API responses. Runtime side effects remain in shared runtime/bridge/policy paths.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `ruse-desktop` and Tauri modules, React 18, TypeScript, Vitest, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M60-01-workspace-dashboard-session.spec.md`
- Read: `docs/specs/M60-desktop-daily-driver.spec.md`
- Read: `docs/plans/M60-desktop-daily-driver.plan.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/ruse-desktop/src/snapshot.rs`
- Read: `crates/ruse-desktop/src/session.rs`
- Read: `crates/ruse-desktop/src/task.rs`
- Read: `crates/ruse-desktop/src/settings.rs`
- Read: `apps/desktop/src/App.tsx`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src/app.css`
- Read: `apps/desktop/src-tauri/src/workspace_snapshot.rs`
- Read: `apps/desktop/src-tauri/src/bridge.rs`

- [x] **Step 1: Review current Desktop dashboard/session baseline**

Run:

```bash
cargo test -p ruse-desktop desktop_shell
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml workspace_snapshot
npm --prefix apps/desktop test -- app-shell
```

Expected: current Desktop session, workspace snapshot, and shell tests pass before dashboard changes.

Baseline verification passed during M60-01:

- `cargo test -p ruse-desktop desktop_shell`
- `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml workspace_snapshot`
- `npm --prefix apps/desktop test -- app-shell`

### Task 2: Add Failing Dashboard Contract Tests

**Files:**
- Create: `crates/ruse-desktop/tests/desktop_dashboard.rs`

- [x] **Step 1: Add serialization and aggregation tests**

Cover:

- connected dashboard serializes stable snake_case readiness
- task counts group active, waiting approval, failed, and terminal states from typed `TaskState`
- provider readiness derives selected provider/model and enabled state from config/settings input
- empty/not-started dashboard is valid
- diagnostics are bounded/redacted and reject empty code/message fields
- serialized dashboard excludes raw prompt, tool input, command output, and secret-like values

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
cargo test -p ruse-desktop --test desktop_dashboard
```

Expected: tests fail before the dashboard contract exists.

Confirmed failing before implementation because the dashboard exports did not exist.

### Task 3: Implement `ruse-desktop` Dashboard Model

**Files:**
- Create: `crates/ruse-desktop/src/dashboard.rs`
- Modify: `crates/ruse-desktop/src/lib.rs`
- Modify: `crates/ruse-desktop/src/session.rs`
- Modify: `crates/ruse-desktop/src/settings.rs`
- Modify: `crates/ruse-desktop/tests/desktop_dashboard.rs`

- [x] **Step 1: Add dashboard contracts**

Implement typed structs/enums for dashboard snapshot, session readiness, provider readiness, task counts, diagnostics, and severity.

- [x] **Step 2: Add pure builders**

Implement helpers that derive dashboard state from workspace snapshot, session readiness inputs, provider/settings snapshot, pending approval count, and diagnostics.

- [x] **Step 3: Run focused contract tests**

Run:

```bash
cargo test -p ruse-desktop --test desktop_dashboard
```

Expected: dashboard contract tests pass.

Passed with six dashboard contract tests.

### Task 4: Add Tauri Dashboard Boundary

**Files:**
- Create: `apps/desktop/src-tauri/src/dashboard.rs`
- Create: `apps/desktop/src-tauri/src/tests/dashboard.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Modify: `apps/desktop/src-tauri/src/workspace_snapshot.rs`

- [x] **Step 1: Add dashboard helper/command**

Expose a typed read-only dashboard snapshot helper or command that adapts the active bridge session, workspace snapshot, config snapshot, settings, and diagnostics into `ruse-desktop` dashboard contracts.

- [x] **Step 2: Add Tauri tests**

Cover not-started, connected, disconnected, and invalid snapshot diagnostics.

- [x] **Step 3: Run focused Tauri tests**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml dashboard
```

Expected: dashboard Tauri tests pass.

Passed with dashboard and bootstrap Tauri coverage.

### Task 5: Add React Dashboard Route

**Files:**
- Create: `apps/desktop/src/components/WorkspaceDashboard.tsx`
- Create: `apps/desktop/src/__tests__/workspace-dashboard.test.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/api.ts`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/app.css`

- [x] **Step 1: Add TypeScript dashboard types/API**

Mirror the Rust/Tauri dashboard response shape in `types.ts` and expose a typed API call in `api.ts`.

- [x] **Step 2: Render dashboard route**

Add a dashboard navigation route with readiness summary, provider status, task counts, pending approvals, diagnostics, and disabled action states when no session is connected.

- [x] **Step 3: Add frontend tests**

Cover route rendering, counts, provider status, diagnostics, empty state, and disconnected/blocked disabled states.

- [x] **Step 4: Run focused frontend tests**

Run:

```bash
npm --prefix apps/desktop test -- workspace-dashboard
```

Expected: dashboard Vitest coverage passes.

Passed with workspace dashboard, app shell, desktop smoke, full frontend test, and frontend build coverage.

### Task 6: Verify Slice and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M60-desktop-daily-driver.plan.md`
- Modify: `docs/plans/M60-01-workspace-dashboard-session.plan.md`

- [x] **Step 1: Run focused checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-desktop --all-targets -- -D warnings
cargo test -p ruse-desktop --test desktop_dashboard
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml dashboard
npm --prefix apps/desktop test -- workspace-dashboard
```

Expected: all focused M60-01 checks pass.

Verification passed:

- `cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m60 cargo clippy -p ruse-desktop --all-targets -- -D warnings`
- `CARGO_TARGET_DIR=target/codex-m60 cargo test -p ruse-desktop desktop_shell`
- `CARGO_TARGET_DIR=target/codex-m60 cargo test -p ruse-desktop --test desktop_dashboard`
- `CARGO_TARGET_DIR=target/codex-m60-tauri cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml dashboard`
- `npm --prefix apps/desktop test`
- `npm --prefix apps/desktop run build`

- [x] **Step 2: Run status and exact-path commit**

Run:

```bash
git status --short --branch
git add docs/specs/M60-01-workspace-dashboard-session.spec.md \
  docs/plans/M60-01-workspace-dashboard-session.plan.md \
  docs/plans/M60-desktop-daily-driver.plan.md \
  docs/milestones/README.md \
  crates/ruse-desktop/src/dashboard.rs \
  crates/ruse-desktop/src/lib.rs \
  crates/ruse-desktop/src/session.rs \
  crates/ruse-desktop/src/settings.rs \
  crates/ruse-desktop/tests/desktop_dashboard.rs \
  apps/desktop/src-tauri/src/dashboard.rs \
  apps/desktop/src-tauri/src/commands.rs \
  apps/desktop/src-tauri/src/lib.rs \
  apps/desktop/src-tauri/src/state.rs \
  apps/desktop/src-tauri/src/workspace_snapshot.rs \
  apps/desktop/src-tauri/src/tests/dashboard.rs \
  apps/desktop/src/App.tsx \
  apps/desktop/src/api.ts \
  apps/desktop/src/types.ts \
  apps/desktop/src/app.css \
  apps/desktop/src/components/WorkspaceDashboard.tsx \
  apps/desktop/src/__tests__/workspace-dashboard.test.tsx
git commit -m "feat: add m60 workspace dashboard"
```

Expected: commit includes only M60-01 docs, source, tests, and status updates.

### Completion Checklist

- [x] Dashboard view models are typed, serde-friendly, bounded, and redacted.
- [x] Session readiness covers not-started, connecting, connected, disconnected, and blocked.
- [x] Task counts derive from typed `TaskState` values.
- [x] Provider readiness exposes selected provider/model without secrets.
- [x] Tauri dashboard boundary is read-only and typed.
- [x] React dashboard route renders readiness, counts, provider status, pending approvals, and diagnostics.
- [x] Focused Rust, Tauri, and frontend tests pass.
