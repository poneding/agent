# M21-01 Tauri Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a runnable Tauri Desktop shell with safe bootstrap diagnostics and no runtime side effects.

**Architecture:** `apps/desktop` owns the GUI and Tauri command layer. `crates/ruse-desktop` remains view-model contracts, and runtime crates stay behind later bridge commands.

**Tech Stack:** Rust 2024 edition, Tauri 2, TypeScript, Vite, Vitest, Testing Library, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M21-01-tauri-shell.spec.md`
- Read: `docs/specs/M21-00-desktop-gui-technical-design.spec.md`
- Read: `crates/ruse-desktop/src/session.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`

- [x] **Step 1: Review shell boundaries**

Run:

```bash
sed -n '1,260p' docs/specs/M21-01-tauri-shell.spec.md
```

Expected: the shell owns presentation and bootstrap only; no model/tool/storage calls are introduced.

### Task 2: Write RED Tests

**Files:**
- Create: `apps/desktop/src-tauri/src/tests/bootstrap.rs`
- Create: `apps/desktop/src/__tests__/app-shell.test.tsx`

- [x] **Step 1: Add Rust bootstrap tests**

Cover bootstrap app name/version, `not_started` runtime state, default route, and redaction of secret-like diagnostic text.

- [x] **Step 2: Add frontend shell tests**

Cover left rail, transcript/composer area, right panel, status bar, bootstrap loading state, and bootstrap error state.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bootstrap
npm --prefix apps/desktop test -- --run app-shell
```

Expected: commands fail because the Tauri app scaffold and bootstrap contracts do not exist.

### Task 3: Implement Tauri Shell

**Files:**
- Create: `apps/desktop/package.json`
- Create: `apps/desktop/index.html`
- Create: `apps/desktop/vite.config.ts`
- Create: `apps/desktop/tsconfig.json`
- Create: `apps/desktop/src/main.tsx`
- Create: `apps/desktop/src/App.tsx`
- Create: `apps/desktop/src/app.css`
- Create: `apps/desktop/src/api.ts`
- Create: `apps/desktop/src/types.ts`
- Create: `apps/desktop/src-tauri/Cargo.toml`
- Create: `apps/desktop/src-tauri/tauri.conf.json`
- Create: `apps/desktop/src-tauri/src/main.rs`
- Create: `apps/desktop/src-tauri/src/lib.rs`
- Create: `apps/desktop/src-tauri/src/bootstrap.rs`
- Create: `apps/desktop/src-tauri/src/commands.rs`
- Create: `apps/desktop/src-tauri/src/state.rs`
- Modify: `Cargo.toml`
- Modify: `README.md`

- [x] **Step 1: Add package and Tauri scaffold**

Create scripts:

```json
{
  "scripts": {
    "dev": "vite --host 127.0.0.1",
    "build": "tsc --noEmit && vite build",
    "test": "vitest run",
    "tauri": "tauri"
  }
}
```

Expected: frontend commands are local to `apps/desktop`.

- [x] **Step 2: Add Rust bootstrap contracts**

Implement `DesktopBootstrap`, `DesktopRuntimeState`, `DesktopBootstrapDiagnostic`, `DesktopAppState`, and `desktop_bootstrap`.

- [x] **Step 3: Add shell frontend**

Render left rail, main area, right panel, composer placeholder, and status bar from bootstrap state.

- [x] **Step 4: Register app crate in workspace**

Add `apps/desktop/src-tauri` as a workspace member and keep `crates/ruse-desktop` as a library dependency.

### Task 4: Verify

**Files:**
- Read: `apps/desktop/src-tauri/src/tests/bootstrap.rs`
- Read: `apps/desktop/src/__tests__/app-shell.test.tsx`

- [x] **Step 1: Run focused Rust tests**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bootstrap
```

Expected: bootstrap tests pass.

- [x] **Step 2: Run frontend tests and build**

Run:

```bash
npm --prefix apps/desktop test -- --run app-shell
npm --prefix apps/desktop run build
```

Expected: shell tests and frontend build pass.

- [x] **Step 3: Run workspace check**

Run:

```bash
cargo xtask check
```

Expected: Rust workspace checks pass.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M21-01-tauri-shell.plan.md`
- Modify: `docs/plans/M21-desktop-gui-mvp.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Mark M21-01 complete**

Update milestone status and set focus to M21-02.
