# M65-03 Desktop Bundle and Update Smoke Plan

> **For agentic workers:** Keep update validation in `agent-desktop` and release orchestration in `xtask`. Do not add Desktop UI redesign work in this slice.

**Goal:** Add beta-grade Desktop package/update smoke evidence with release-local roots, stable smoke check ids, Tauri metadata validation, update manifest evidence, focused tests, and docs.

**Architecture:** `xtask` plans Desktop bundle/update smoke; `agent-desktop` validates update manifests; Tauri owns bundle metadata.

**Tech Stack:** Rust 2024 edition, Tauri 2 config, existing `xtask` release structs, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M65-03-desktop-bundle-update-smoke.spec.md`
- Read: `docs/specs/M25-05-desktop-bundle-updater.spec.md`
- Read: `xtask/src/lib.rs`
- Read: `xtask/tests/package_desktop.rs`
- Read: `crates/agent-desktop/src/update.rs`
- Read: `crates/agent-desktop/tests/update_manifest.rs`
- Read: `apps/desktop/src-tauri/tauri.conf.json`

- [x] **Step 1: Review existing Desktop packaging path**

Expected: M25-05 already validates Tauri metadata, writes a plan artifact/update manifest, and runs update/settings smoke checks.

### Task 2: Add Failing Tests

**Files:**
- Modify: `xtask/tests/package_desktop.rs`

- [x] **Step 1: Assert Desktop smoke summary**

Add assertions for smoke root, Tauri config path, bundle requested state, and stable check ids.

- [x] **Step 2: Assert plan/update evidence remains deterministic**

Keep existing deterministic plan artifact and update manifest path coverage.

- [x] **Step 3: Run focused tests**

```bash
CARGO_TARGET_DIR=target/m65-dev cargo test -p xtask --test package_desktop
```

Expected: new Desktop smoke summary assertions cover the new evidence.

### Task 3: Implement Desktop Release Smoke Evidence

**Files:**
- Modify: `xtask/src/lib.rs`

- [x] **Step 1: Add Desktop smoke summary contracts**

Add additive structs or helper functions for smoke root, Tauri config path, bundle requested state, and smoke check metadata.

- [x] **Step 2: Populate summary from package plan**

When `run_package_desktop` succeeds, include summary evidence in the plan artifact or companion report without raw command output.

- [x] **Step 3: Preserve update manifest focus**

Keep `DesktopUpdateManifest` focused on update metadata: channel, version, artifacts, URL, checksum, and optional signature.

### Task 4: Update Docs And Status

**Files:**
- Create or modify: `docs/desktop/release-packaging.md`
- Modify: `docs/release/beta-gate.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M65-packaging-release-beta-gate.plan.md`
- Modify: `docs/plans/M65-03-desktop-bundle-update-smoke.plan.md`

- [x] **Step 1: Document Desktop package/update smoke**

Explain command, bundle-vs-plan behavior, update manifest, smoke root, Tauri metadata checks, and privacy guarantees.

- [x] **Step 2: Update M65 status**

Mark M65-03 complete only after focused tests and docs pass. Set M65-04 as next active slice.

### Task 5: Verify

- [x] **Step 1: Run focused gates**

```bash
CARGO_TARGET_DIR=target/m65-dev cargo test -p xtask --test package_desktop
CARGO_TARGET_DIR=target/m65-dev cargo test -p agent-desktop update_manifest
CARGO_TARGET_DIR=target/m65-dev cargo clippy -p xtask -p agent-desktop --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 2: Optional package smoke**

Run when local time permits:

```bash
cargo xtask package-desktop --channel dev
```

Expected: Desktop plan artifact and update manifest are written under `target/agent-release/`.

### Task 6: Commit

**Files:**
- M65-03 spec/plan, `xtask` code/tests, Desktop release docs, milestone/roadmap status.

- [x] **Step 1: Commit implementation**

Suggested commit message:

```text
feat: add m65 desktop package update smoke
```
