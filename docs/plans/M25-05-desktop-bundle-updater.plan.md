# M25-05 Desktop Bundle and Updater Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Desktop bundle packaging, update manifest validation, and local Desktop release smoke checks.

**Architecture:** `xtask` orchestrates packaging, Tauri owns bundle config, and `agent-desktop` owns updater-safe contracts.

**Tech Stack:** Rust 2024 edition, Tauri bundle config, update manifest schema, xtask command planning, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M25-05-desktop-bundle-updater.spec.md`
- Read: `apps/desktop/src-tauri/tauri.conf.json`
- Read: `apps/desktop/package.json`
- Read: `crates/agent-desktop/src/lib.rs`

- [x] **Step 1: Review Desktop packaging scope**

Expected: bundle metadata, update manifest contracts, smoke checks, and local failure behavior are clear.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-desktop/tests/update_manifest.rs`
- Modify: `xtask/tests/xtask_plan.rs`
- Create if needed: `xtask/tests/package_desktop.rs`

- [x] **Step 1: Add update manifest tests**

Cover valid manifest, invalid checksum, unsupported channel, and missing artifact fields.

- [x] **Step 2: Add xtask Desktop packaging tests**

Cover command planning for local Desktop bundle and smoke checks.

- [x] **Step 3: Verify RED**

```bash
cargo test -p agent-desktop update_manifest
cargo test -p xtask package_desktop
```

### Task 3: Implement Desktop Packaging and Updater Contracts

**Files:**
- Modify: `crates/agent-desktop/src/lib.rs`
- Create if needed: `crates/agent-desktop/src/update.rs`
- Modify: `xtask/src/lib.rs`
- Modify: `xtask/src/main.rs`
- Modify if needed: `apps/desktop/src-tauri/tauri.conf.json`

- [x] **Step 1: Add update manifest contracts**

Validate channel, version, platform, checksum, URL, and optional signature.

- [x] **Step 2: Add Desktop package command**

Plan Tauri bundle generation and smoke checks without embedding local state.

- [x] **Step 3: Add local update failure tests**

Ensure invalid manifests and missing artifacts preserve current app state.

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p agent-desktop update_manifest
cargo test -p xtask package_desktop
```

- [x] **Step 2: Run M25 release smoke checks**

```bash
cargo xtask package-desktop --channel dev
```

- [x] **Step 3: Run workspace gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M25-05-desktop-bundle-updater.plan.md`
- Modify: `docs/plans/M25-packaging-release-settings-updates.plan.md`

- [x] **Step 1: Mark M25-05 and M25 complete**

Record final release-readiness status.

- [x] **Step 2: Commit M25 completion**

```bash
git add crates xtask apps docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m25 packaging release"
```
