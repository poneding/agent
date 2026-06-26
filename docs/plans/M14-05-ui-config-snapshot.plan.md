# M14-05 UI Config Snapshot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add UI bridge, CLI JSON, and Desktop config snapshot contracts.

**Architecture:** Convert effective runtime config into UI-safe snapshot types in `agent-ui-bridge`; CLI JSON can emit those snapshots, and `agent-desktop` can consume them without loading config files or resolving secrets.

**Tech Stack:** Rust 2024 edition, serde, serde_json snapshot tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M14-05-ui-config-snapshot.spec.md`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-desktop/src/lib.rs`
- Read: `crates/agent-desktop/tests/settings.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M14-05-ui-config-snapshot.spec.md
```

Expected: the spec defines UI snapshot contracts, state handling, and privacy tests.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-ui-bridge/tests/config_snapshot.rs`
- Create: `crates/agent-desktop/tests/config_snapshot.rs`
- Modify: `crates/agent-cli/tests/config_diagnostics.rs`

- [x] **Step 1: Add focused tests**

Add UI bridge tests:

- `config_snapshot_serializes_stable_shape`
- `config_snapshot_keeps_disabled_providers_visible`
- `config_snapshot_redacts_secret_like_values`
- `config_snapshot_json_frame_is_single_line`

Add Desktop tests:

- `desktop_config_snapshot_rejects_empty_selection`
- `desktop_config_snapshot_accepts_ui_bridge_snapshot`

Extend CLI config diagnostics tests so JSON config-check output includes `UiJsonFrame::ConfigSnapshot`.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-ui-bridge --test config_snapshot
cargo test -p agent-desktop --test config_snapshot
```

Expected: tests fail because snapshot contracts do not exist.

### Task 3: Implement Snapshot Contracts

**Files:**
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-desktop/src/lib.rs`

- [x] **Step 1: Add UI bridge snapshot types**

Implement:

- `UiConfigSnapshot`
- `UiModelProviderSummary`
- `UiConfigDiagnostic`
- `UiConfigDiagnosticSeverity`
- `UiJsonFrame::ConfigSnapshot`

- [x] **Step 2: Emit CLI snapshot frame**

Convert effective config-check output into `UiConfigSnapshot` and emit `UiJsonFrame::ConfigSnapshot` in JSON mode.

- [x] **Step 3: Add Desktop wrapper**

Implement a small Desktop view model that validates selected provider/model and wraps `UiConfigSnapshot`.

- [x] **Step 4: Verify focused tests**

Run:

```bash
cargo test -p agent-ui-bridge --test config_snapshot
cargo test -p agent-desktop --test config_snapshot
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M14-05-ui-config-snapshot.plan.md`
- Modify: `docs/plans/M14-runtime-configuration-secrets.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m14 ui config snapshot"
```

Expected: commit succeeds.
