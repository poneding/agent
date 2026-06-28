# M25-01 Platform Paths and Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add shared platform path resolution, typed settings profiles, safe settings mutation, and CLI/UI diagnostics.

**Architecture:** `agent-core` owns contracts, `agent-storage` handles atomic persistence, `agent-cli` exposes commands, and `agent-ui-bridge` carries safe snapshots.

**Tech Stack:** Rust 2024 edition, serde, platform path fixtures, atomic file writes, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M25-01-platform-paths-settings.spec.md`
- Read: `docs/specs/M25-00-packaging-release-technical-design.spec.md`
- Read: `docs/architecture/configuration.md`
- Read: `crates/agent-core/src/runtime_config.rs`

- [x] **Step 1: Review path and settings scope**

Expected: path ownership, settings mutation, diagnostics, and persistence boundaries are clear.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-core/tests/platform_paths_settings.rs`
- Create if needed: `crates/agent-cli/tests/settings_paths.rs`
- Create if needed: `crates/agent-ui-bridge/tests/settings_paths_snapshot.rs`

- [x] **Step 1: Add path fixture tests**

Cover macOS, Linux, Windows, overrides, and portable install mode.

- [x] **Step 2: Add settings mutation tests**

Cover validation, atomic persistence, corrupted file diagnostics, and redaction.

- [x] **Step 3: Verify RED**

```bash
cargo test -p agent-core platform_paths_settings
```

### Task 3: Implement Contracts

**Files:**
- Modify: `crates/agent-core/src/lib.rs`
- Create if needed: `crates/agent-core/src/platform.rs`
- Create if needed: `crates/agent-core/src/settings.rs`
- Modify if needed: `crates/agent-storage/src/lib.rs`
- Modify if needed: `crates/agent-cli/src/command.rs`
- Modify if needed: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Add `PlatformPaths`**

Implement deterministic platform resolution with explicit overrides.

- [x] **Step 2: Add `SettingsProfile` and mutations**

Validate settings changes before persistence and keep secret values out of config.

- [x] **Step 3: Add diagnostics and snapshots**

Expose safe text/JSON outputs for CLI and UI clients.

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p agent-core platform_paths_settings
cargo test -p agent-cli settings_paths
cargo test -p agent-ui-bridge settings_paths_snapshot
```

- [x] **Step 2: Run workspace gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M25-01-platform-paths-settings.plan.md`
- Modify: `docs/plans/M25-packaging-release-settings-updates.plan.md`

- [x] **Step 1: Mark M25-01 complete**

Advance active focus to M25-02.

- [x] **Step 2: Commit M25-01**

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: add platform paths and settings"
```
