# M10-01 Desktop Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `ruse-desktop` with typed Desktop session and shell route contracts.

**Architecture:** `ruse-desktop` depends on `ruse-types` and `ruse-ui-bridge` only in this slice. Runtime execution remains in `ruse-core`; this crate exposes presentation contracts that a future native Desktop UI can consume.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `thiserror`, existing `ruse-types`, `ruse-ui-bridge`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold `ruse-desktop`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-desktop/Cargo.toml`
- Create: `crates/ruse-desktop/src/lib.rs`
- Create: `crates/ruse-desktop/src/error.rs`

- [x] **Step 1: Add crate scaffold**

Add `crates/ruse-desktop` to workspace members.

Create the crate with dependencies:

```toml
[dependencies]
ruse-types = { path = "../ruse-types" }
ruse-ui-bridge = { path = "../ruse-ui-bridge" }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

### Task 2: Write Desktop Shell RED Tests

**Files:**
- Create: `crates/ruse-desktop/tests/desktop_shell.rs`

- [x] **Step 1: Add failing tests**

Cover:

- `desktop_shell_serializes_session_shape`
- `desktop_shell_starts_on_conversation_route`
- `desktop_shell_switches_routes`
- `desktop_shell_rejects_empty_client_version`
- `desktop_shell_rejects_route_switch_after_disconnect`

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-desktop desktop_shell
```

Expected: fails because `ruse-desktop` shell contracts do not exist.

### Task 3: Implement Desktop Shell Contracts

**Files:**
- Create: `crates/ruse-desktop/src/session.rs`
- Modify: `crates/ruse-desktop/src/lib.rs`

- [x] **Step 1: Add session contracts**

Implement `DesktopSessionState`, `DesktopRoute`, and `DesktopSession`.

- [x] **Step 2: Add session transitions**

Implement `start_desktop_session`, `switch_desktop_route`, and `disconnect_desktop_session`.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p ruse-desktop desktop_shell
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M10-01-desktop-shell.plan.md`

- [x] **Step 1: Advance active focus**

Set active focus to `M10-02-conversation-tasks`.

- [x] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m10 desktop shell"
```
