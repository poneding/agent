# M22-05 Extension Health UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose extension registry and health state through UI-safe bridge contracts, CLI commands, TUI panel data, and Desktop view models.

**Architecture:** `agent-ui-bridge` owns serializable extension snapshots. `agent-cli`, `agent-tui`, `agent-desktop`, and `apps/desktop` render them without owning extension mutation.

**Tech Stack:** Rust 2024 edition, serde, Ratatui render tests, React/Vitest, Cargo fmt, Cargo clippy, Cargo test, npm build/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M22-05-extension-health-ui.spec.md`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-tui/src/trace_panel.rs`
- Read: `crates/agent-desktop/src/settings.rs`
- Read: `apps/desktop/src/App.tsx`

- [x] **Step 1: Review UI health scope**

Run:

```bash
sed -n '1,280p' docs/specs/M22-05-extension-health-ui.spec.md
```

Expected: UI bridge contracts, CLI/TUI/Desktop surfaces, and redaction requirements are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-ui-bridge/tests/extension_snapshot.rs`
- Create: `crates/agent-cli/tests/extension_health_commands.rs`
- Create: `crates/agent-tui/tests/extension_panel.rs`
- Create: `crates/agent-desktop/tests/extension_health.rs`
- Create: `apps/desktop/src/__tests__/extension-health-ui.test.tsx`

- [x] **Step 1: Add bridge snapshot tests**

Cover serialization, deterministic ordering, diagnostics, capability reviews, and redaction.

- [x] **Step 2: Add CLI/TUI tests**

Cover `agent extension list|inspect|health` in text/JSON and compact TUI rendering.

- [x] **Step 3: Add Desktop tests**

Cover extension health component rendering, unhealthy states, review-required states, and secret-safe diagnostics.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-ui-bridge extension_snapshot
cargo test -p agent-cli extension_health_commands
cargo test -p agent-tui extension_panel
cargo test -p agent-desktop extension_health
npm --prefix apps/desktop test -- --run extension-health-ui
```

Expected: tests fail because UI contracts and views are missing.

### Task 3: Implement Extension Health Surfaces

**Files:**
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Create: `crates/agent-tui/src/extension_panel.rs`
- Modify: `crates/agent-tui/src/lib.rs`
- Create: `crates/agent-desktop/src/extension.rs`
- Modify: `crates/agent-desktop/src/lib.rs`
- Create: `apps/desktop/src/components/ExtensionHealthView.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/types.ts`

- [x] **Step 1: Add bridge contracts**

Implement `UiExtensionSnapshot`, `UiExtensionSummary`, `UiExtensionCapabilityReview`, `UiExtensionDiagnostic`, and JSON frame support.

- [x] **Step 2: Add CLI commands**

Render extension list, inspect, and health from registry snapshots in text and JSON modes.

- [x] **Step 3: Add TUI and Desktop view models**

Render compact extension summaries, health diagnostics, and capability review status.

- [x] **Step 4: Add final M22 smoke coverage**

Exercise plugin fixture, skill fixture, MCP fixture, and extension health snapshot together.

### Task 4: Verify

**Files:**
- Read: `crates/agent-ui-bridge/tests/extension_snapshot.rs`
- Read: `apps/desktop/src/__tests__/extension-health-ui.test.tsx`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-ui-bridge extension_snapshot
cargo test -p agent-cli extension_health_commands
cargo test -p agent-tui extension_panel
cargo test -p agent-desktop extension_health
npm --prefix apps/desktop test -- --run extension-health-ui
```

Expected: focused tests pass.

- [x] **Step 2: Run final M22 checks**

Run:

```bash
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask check
```

Expected: frontend tests, frontend build, and workspace checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M22-05-extension-health-ui.plan.md`
- Modify: `docs/plans/M22-extension-ux-mcp-gateway.plan.md`

- [x] **Step 1: Mark M22-05 and M22 complete**

Update milestone status and set next focus to M23.

- [x] **Step 2: Commit M22 completion**

Run:

```bash
git add crates apps docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m22 extension ux"
```

Expected: commit succeeds.
