# M63-04 TUI/Desktop Team Surfaces Plan

> **For agentic workers:** Keep this slice UI-only. Consume bridge snapshots; do not implement readiness logic in TUI or Desktop.

**Goal:** Add TUI and Desktop surfaces for M63 team workflow snapshots.

**Architecture:** `agent-ui-bridge` owns frame shape. `agent-team` owns readiness and merge blockers. TUI/Desktop render typed snapshots only.

---

### Task 1: Plan UI Surfaces

**Files:**
- Create: `docs/specs/M63-04-tui-desktop-team-surfaces.spec.md`
- Create: `docs/plans/M63-04-tui-desktop-team-surfaces.plan.md`

- [x] **Step 1: Define TUI surface**

Render team workflow snapshots in the existing task side panel.

- [x] **Step 2: Define Desktop surface**

Render team workflow snapshots in the existing Tasks view with a focused component.

### Task 2: Implement TUI Rendering

**Files:**
- Modify: `crates/agent-tui/src/task_panel.rs`
- Modify tests under `crates/agent-tui/tests/`

- [x] **Step 1: Reduce team frames**

Store `UiTeamWorkflowSnapshot` values in task panel state when bridge frames arrive.

- [x] **Step 2: Render team summaries**

Show operation, status, readiness, dry-run state, handoffs, blackboard counts, capability ceiling, conflicts, diagnostics, and errors.

- [x] **Step 3: Test panel and app reducer**

Cover direct panel reduction and app-level bridge frame routing.

### Task 3: Implement Desktop Rendering

**Files:**
- Modify: `apps/desktop/src/types.ts`
- Create: `apps/desktop/src/components/TeamWorkflowView.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/app.css`
- Modify tests under `apps/desktop/src/__tests__/`

- [x] **Step 1: Add typed snapshot interfaces**

Mirror the bridge-safe team workflow frame shape in TypeScript.

- [x] **Step 2: Add component**

Render a compact team workflow panel in the Tasks view.

- [x] **Step 3: Test component/AppShell rendering**

Cover readiness, handoffs, conflicts, diagnostics, and empty state.

### Task 4: Verify

**Files:**
- Read: changed files

- [x] **Step 1: Format/build checks**

```bash
cargo fmt --all -- --check
npm --prefix apps/desktop run build
```

- [x] **Step 2: Focused tests**

```bash
CARGO_TARGET_DIR=target/m63-gate cargo test -p agent-tui --test task_panel
npm --prefix apps/desktop test -- --run src/__tests__/team-workflow-ui.test.tsx
```

- [x] **Step 3: Focused clippy**

```bash
CARGO_TARGET_DIR=target/m63-gate cargo clippy -p agent-tui --all-targets -- -D warnings
```

- [x] **Step 4: Diff check**

```bash
git diff --check
```

### Task 5: Status And Commit

**Files:**
- Modify: `docs/plans/M63-worktree-agent-team-workflows.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Update status docs after focused gates pass**

Mark M63-04 implementation complete only after focused gates pass. Keep M63 overall in progress.

- [ ] **Step 2: Commit**

Exact-stage only M63-04 files. Exclude `tmp/`.

Suggested commit message:

```text
feat: add m63 team workflow ui surfaces
```
