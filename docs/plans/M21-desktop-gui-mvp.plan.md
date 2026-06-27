# M21 Desktop GUI MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a runnable Tauri Desktop MVP over the shared runtime bridge.

**Architecture:** Tauri hosts the Desktop frontend and Rust command layer. `agent-desktop` remains view-model contracts, `agent-ui-bridge` carries commands/events, and runtime crates stay UI-agnostic.

**Tech Stack:** Rust 2024 edition, Tauri 2, TypeScript frontend, UI bridge schemas, Playwright smoke tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M21 Technical Design

**Files:**
- Create: `docs/specs/M21-00-desktop-gui-technical-design.spec.md`
- Create: `docs/plans/M21-00-desktop-gui-technical-design.plan.md`

- [x] **Step 1: Write the M21 technical design baseline**

Define Tauri app layout, Rust command boundary, frontend state model, bridge event flow, settings, security, and smoke tests.

- [x] **Step 2: Write the M21 design execution plan**

Document frontend and Rust slices with separate verification commands.

### Task 2: Decompose M21 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M21-01-tauri-shell.spec.md`
- Create: `docs/plans/M21-01-tauri-shell.plan.md`
- Create: `docs/specs/M21-02-desktop-bridge-session.spec.md`
- Create: `docs/plans/M21-02-desktop-bridge-session.plan.md`
- Create: `docs/specs/M21-03-conversation-task-ui.spec.md`
- Create: `docs/plans/M21-03-conversation-task-ui.plan.md`
- Create: `docs/specs/M21-04-approval-diff-ui.spec.md`
- Create: `docs/plans/M21-04-approval-diff-ui.plan.md`
- Create: `docs/specs/M21-05-trace-settings-ui.spec.md`
- Create: `docs/plans/M21-05-trace-settings-ui.plan.md`

- [x] **Step 1: Create slice specs**

Each slice covers Rust commands, frontend state, privacy, and test expectations.

- [x] **Step 2: Create slice plans**

Each plan includes focused Rust tests and frontend smoke tests.

### Task 3: Implement M21 Slices in Order

**Files:**
- Execute: `docs/plans/M21-01-tauri-shell.plan.md`
- Execute: `docs/plans/M21-02-desktop-bridge-session.plan.md`
- Execute: `docs/plans/M21-03-conversation-task-ui.plan.md`
- Execute: `docs/plans/M21-04-approval-diff-ui.plan.md`
- Execute: `docs/plans/M21-05-trace-settings-ui.plan.md`

- [x] **Step 1: Add runnable Tauri shell**

Create the app scaffold and startup diagnostics without runtime side effects.

- [x] **Step 2: Connect bridge session**

Wire typed commands and event streaming between frontend and runtime.

- [x] **Step 3: Build conversation and task views**

Render conversations, composer, and task list from view models.

- [x] **Step 4: Build approval and diff views**

Render approval requests and diff reviews using shared contracts.

- [ ] **Step 5: Build trace and settings views**

Render trace timeline and validate settings without secrets.

### Task 4: Complete M21

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M21-desktop-gui-mvp.plan.md`

- [ ] **Step 1: Run final verification**

Run:

```bash
cargo xtask check
```

Expected: Rust workspace checks pass. Frontend smoke command from the M21 slice plan also passes.

- [ ] **Step 2: Mark M21 complete**

Update milestone status and advance active focus to M22.

- [ ] **Step 3: Commit M21 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m21 desktop mvp"
```

Expected: commit succeeds.
