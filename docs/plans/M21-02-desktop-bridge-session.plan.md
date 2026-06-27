# M21-02 Desktop Bridge Session Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect Desktop to the shared UI bridge through typed Tauri commands and bounded UI-safe frame delivery.

**Architecture:** Tauri commands own `DesktopRuntimeSession`; frontend calls only typed command wrappers. `agent-ui-bridge` remains the durable UI command/event schema boundary.

**Tech Stack:** Rust 2024 edition, Tauri 2 commands, `agent-ui-bridge`, `agent-desktop`, TypeScript, Vitest, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M21-02-desktop-bridge-session.spec.md`
- Read: `apps/desktop/src-tauri/src/state.rs`
- Read: `crates/agent-ui-bridge/src/command.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Review bridge contracts**

Run:

```bash
sed -n '1,260p' docs/specs/M21-02-desktop-bridge-session.spec.md
```

Expected: frontend receives only UI bridge frames and cannot call runtime internals.

### Task 2: Write RED Tests

**Files:**
- Create: `apps/desktop/src-tauri/src/tests/bridge_session.rs`
- Create: `apps/desktop/src/__tests__/bridge-session.test.ts`

- [x] **Step 1: Add Rust session tests**

Cover start, duplicate start, stop, idempotent stop, command validation, bounded frame draining, and redacted diagnostics.

- [x] **Step 2: Add frontend bridge client tests**

Cover `startSession`, `stopSession`, `sendUiCommand`, `drainUiFrames`, reconnect state, and command error rendering.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bridge_session
npm --prefix apps/desktop test -- --run bridge-session
```

Expected: tests fail because bridge session contracts are missing.

### Task 3: Implement Bridge Session

**Files:**
- Create: `apps/desktop/src-tauri/src/bridge.rs`
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/state.rs`
- Modify: `apps/desktop/src/api.ts`
- Create: `apps/desktop/src/bridge.ts`
- Modify: `apps/desktop/src/types.ts`

- [x] **Step 1: Add session state**

Implement `DesktopRuntimeSession`, `DesktopStartSessionRequest`, `DesktopSessionSnapshot`, `DesktopCommandAck`, and bounded in-memory frame queue.

- [x] **Step 2: Add Tauri commands**

Implement `start_session`, `stop_session`, `send_ui_command`, and `drain_ui_frames`.

- [x] **Step 3: Add frontend client**

Wrap command calls in `apps/desktop/src/bridge.ts` and surface typed errors to UI state.

- [x] **Step 4: Redact bridge diagnostics**

Ensure command errors and queued diagnostics redact secret-like tokens before serialization.

### Task 4: Verify

**Files:**
- Read: `apps/desktop/src-tauri/src/tests/bridge_session.rs`
- Read: `apps/desktop/src/__tests__/bridge-session.test.ts`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml bridge_session
npm --prefix apps/desktop test -- --run bridge-session
```

Expected: bridge session tests pass.

- [x] **Step 2: Run shell regression tests**

Run:

```bash
npm --prefix apps/desktop test
cargo xtask check
```

Expected: Desktop frontend tests and Rust workspace checks pass.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M21-02-desktop-bridge-session.plan.md`
- Modify: `docs/plans/M21-desktop-gui-mvp.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Mark M21-02 complete**

Update milestone status and set focus to M21-03.
