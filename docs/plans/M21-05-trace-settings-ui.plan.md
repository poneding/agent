# M21-05 Trace and Settings UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Desktop trace timeline, secret-safe settings view, and final Desktop MVP smoke coverage.

**Architecture:** Tauri commands return `TraceTimeline`, `DesktopConfigSnapshot`, and settings validation diagnostics. Frontend renders safe summaries and defers durable settings mutation to M25.

**Tech Stack:** Rust 2024 edition, Tauri 2 commands, `ruse-desktop`, `ruse-ui-bridge`, TypeScript, React, Vitest, Testing Library, Playwright smoke test, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M21-05-trace-settings-ui.spec.md`
- Read: `crates/ruse-desktop/src/trace.rs`
- Read: `crates/ruse-desktop/src/settings.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `docs/plans/M21-desktop-gui-mvp.plan.md`

- [x] **Step 1: Review trace/settings scope**

Run:

```bash
sed -n '1,260p' docs/specs/M21-05-trace-settings-ui.spec.md
```

Expected: M21 validates and displays settings only; durable settings mutation remains M25.

### Task 2: Write RED Tests

**Files:**
- Create: `apps/desktop/src-tauri/src/tests/trace_settings_commands.rs`
- Create: `apps/desktop/src/__tests__/trace-settings-ui.test.tsx`
- Create: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`

- [x] **Step 1: Add Rust command tests**

Cover empty trace, populated trace, duplicate sequence diagnostics, config snapshot, invalid settings, and secret redaction.

- [x] **Step 2: Add frontend tests**

Cover trace severity rendering, empty trace, settings fields, invalid settings diagnostic, and redacted secret-like values.

- [x] **Step 3: Add smoke navigation test**

Cover conversation, tasks, approval, trace, and settings route navigation.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml trace_settings_commands
npm --prefix apps/desktop test -- --run trace-settings-ui
npm --prefix apps/desktop test -- --run desktop-smoke
```

Expected: tests fail because trace/settings commands and views are missing.

### Task 3: Implement Trace and Settings UI

**Files:**
- Create: `apps/desktop/src-tauri/src/trace_commands.rs`
- Create: `apps/desktop/src-tauri/src/settings_commands.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Create: `apps/desktop/src/state/trace.ts`
- Create: `apps/desktop/src/state/settings.ts`
- Create: `apps/desktop/src/components/TraceTimeline.tsx`
- Create: `apps/desktop/src/components/SettingsView.tsx`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/api.ts`
- Modify: `apps/desktop/src/types.ts`

- [x] **Step 1: Add trace command**

Implement `load_trace` using `ruse-desktop::build_trace_timeline` and fixture-safe empty trace fallback.

- [x] **Step 2: Add config/settings commands**

Implement `load_config_snapshot` and `validate_settings` with redacted diagnostics and no durable mutation.

- [x] **Step 3: Add trace/settings reducers**

Represent loading, loaded, empty, and error states for trace and settings views.

- [x] **Step 4: Add components and navigation**

Render trace timeline, settings sections, validation diagnostics, and route navigation across the Desktop MVP.

### Task 4: Verify

**Files:**
- Read: `apps/desktop/src-tauri/src/tests/trace_settings_commands.rs`
- Read: `apps/desktop/src/__tests__/trace-settings-ui.test.tsx`
- Read: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml trace_settings_commands
npm --prefix apps/desktop test -- --run trace-settings-ui
npm --prefix apps/desktop test -- --run desktop-smoke
```

Expected: focused tests pass.

- [x] **Step 2: Run final Desktop checks**

Run:

```bash
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask check
```

Expected: Desktop frontend tests, frontend build, and Rust workspace checks pass.

### Task 5: Update Status

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M21-05-trace-settings-ui.plan.md`
- Modify: `docs/plans/M21-desktop-gui-mvp.plan.md`

- [x] **Step 1: Mark M21-05 and M21 complete**

Update milestone status and set next focus to M22.
