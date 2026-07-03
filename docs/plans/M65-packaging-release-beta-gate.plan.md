# M65 Packaging, Release, and Beta Gate Implementation Plan

> **For agentic workers:** Keep release logic in `xtask` and runtime crates. CLI/Desktop should render typed diagnostics and release evidence; they should not own migration, provider, secret, update, or support-bundle policy.

**Goal:** Prepare a beta users can install and trust by productizing CLI/Desktop packaging, isolated release smokes, migration/secret/provider diagnostics, support bundles, and durable beta gate evidence.

**Architecture:** `xtask` orchestrates release gates; `agent-core`, `agent-model`, `agent-storage`, `agent-desktop`, and `agent-ui-bridge` own typed runtime diagnostics; CLI/Desktop render bounded summaries.

**Tech Stack:** Rust 2024 edition, Cargo workspace, Tauri 2, Vite/React Desktop app, `xtask`, Cargo fmt, Cargo clippy, Cargo test, Vitest.

---

### Task 1: Confirm Baseline

**Files:**
- Read: `docs/specs/M65-packaging-release-beta-gate.spec.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `docs/specs/M25-packaging-release-settings-updates.spec.md`
- Read: `docs/specs/M29-ci-quality-evidence-release-gates.spec.md`
- Read: `xtask/src/lib.rs`
- Read: `xtask/tests/package_cli.rs`
- Read: `xtask/tests/package_desktop.rs`
- Read: `crates/agent-core/src/startup_migration.rs`
- Read: `crates/agent-storage/src/migration.rs`
- Read: `crates/agent-model/src/`
- Read: `crates/agent-desktop/src/update.rs`

- [x] **Step 1: Confirm existing packaging foundations**

Existing M25/M29 foundations include `package-cli`, `package-desktop`, update manifest validation, startup migrations, eval gates, Desktop build/test gates, and quality evidence. M65 should productize these foundations rather than rebuild them.

### Task 2: M65-01 Release Baseline and Beta Gate Contracts

**Files:**
- Create: `docs/specs/M65-01-release-baseline-beta-gate.spec.md`
- Create: `docs/plans/M65-01-release-baseline-beta-gate.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Plan M65-01**

Define the beta artifact matrix, gate report shape, support evidence sections, isolated smoke roots, current gap list, and acceptance commands. Keep this slice focused on planning/contracts unless implementation is explicitly started.

- [x] **Step 2: Complete M65-01 planning-only slice**

M65-01 is complete as a planning/contracts baseline. Code implementation begins in M65-02 unless the beta gate contract is pulled forward explicitly.

### Task 3: M65-02 CLI Package and Install Smoke

**Files:**
- Create: `docs/specs/M65-02-cli-package-install-smoke.spec.md`
- Create: `docs/plans/M65-02-cli-package-install-smoke.plan.md`
- Modify: `xtask/src/lib.rs`
- Modify: `xtask/src/main.rs`
- Modify: `xtask/tests/package_cli.rs`
- Modify: `docs/cli/`

- [ ] **Step 1: Plan CLI beta package smoke**

Specify packaged CLI artifact checks, release-local config roots, startup/config/provider diagnostics, manifest/checksum evidence, and failure behavior.

- [ ] **Step 2: Implement CLI package smoke**

Extend existing package CLI commands only as needed for beta-grade install/run evidence.

### Task 4: M65-03 Desktop Bundle and Update Smoke

**Files:**
- Create: `docs/specs/M65-03-desktop-bundle-update-smoke.spec.md`
- Create: `docs/plans/M65-03-desktop-bundle-update-smoke.plan.md`
- Modify: `xtask/src/lib.rs`
- Modify: `xtask/tests/package_desktop.rs`
- Modify: `crates/agent-desktop/src/update.rs`
- Modify if needed: `apps/desktop/src-tauri/tauri.conf.json`
- Modify: `docs/desktop/`

- [ ] **Step 1: Plan Desktop beta bundle smoke**

Specify Tauri metadata validation, update manifest checks, app startup diagnostics, Desktop smoke roots, and bundle-vs-plan behavior.

- [ ] **Step 2: Implement Desktop package smoke**

Keep Desktop smoke deterministic and credential-free. Do not redesign Desktop UI in M65.

### Task 5: M65-04 Settings, Secrets, and Migration Onboarding

**Files:**
- Create: `docs/specs/M65-04-settings-secrets-migration-onboarding.spec.md`
- Create: `docs/plans/M65-04-settings-secrets-migration-onboarding.plan.md`
- Modify: `crates/agent-core/src/startup_migration.rs`
- Modify: `crates/agent-storage/src/migration.rs`
- Modify: `crates/agent-model/`
- Modify: `crates/agent-ui-bridge/`
- Modify: `crates/agent-cli/`
- Modify: `apps/desktop/src/`

- [ ] **Step 1: Plan upgrade/onboarding diagnostics**

Specify migration diagnostics, keychain/env secret setup, provider readiness, config migration safety, and CLI/Desktop rendering.

- [ ] **Step 2: Implement upgrade/onboarding diagnostics**

Runtime crates own decisions. CLI/Desktop render typed snapshots and next actions.

### Task 6: M65-05 Support Bundle and Release Evidence

**Files:**
- Create: `docs/specs/M65-05-support-bundle-release-evidence.spec.md`
- Create: `docs/plans/M65-05-support-bundle-release-evidence.plan.md`
- Modify: `xtask/src/lib.rs`
- Modify: `crates/agent-core/`
- Modify: `crates/agent-telemetry/`
- Modify: `crates/agent-cli/`
- Modify if needed: `crates/agent-ui-bridge/`
- Modify: `docs/`

- [ ] **Step 1: Plan support bundle contracts**

Specify included sections, redaction diagnostics, artifact layout, support commands, and release gate integration.

- [ ] **Step 2: Implement support bundle evidence**

Generate bounded, redacted evidence. Never include raw secrets, raw prompts, sensitive memory, embeddings, or unbounded logs.

### Task 7: M65-06 Beta Gate Docs and Closeout

**Files:**
- Create: `docs/specs/M65-06-beta-gate-docs-closeout.spec.md`
- Create: `docs/plans/M65-06-beta-gate-docs-closeout.plan.md`
- Create or modify: `docs/release/`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify if needed: `README.md`

- [ ] **Step 1: Add release docs**

Document install, upgrade, provider setup, keychain/env setup, package smoke, Desktop bundle/update smoke, support bundle generation, and failure recovery.

- [ ] **Step 2: Run final M65 gates**

Run:

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=target/m65-dev cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=target/m65-dev cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
cargo xtask package-cli --channel dev
cargo xtask package-desktop --channel dev
git diff --check
```

Expected: M65 is packaged, smoke-tested, documented, and ready to hand off to M66.

### Task 8: Commit Discipline

**Files:**
- Modify/create only files for the active slice and required status docs.

- [x] **Step 1: Commit M65 planning**

Exact-stage only M65 milestone planning docs and milestone/roadmap index updates.

Suggested commit message:

```text
docs: add m65 packaging beta gate plan
```

- [ ] **Step 2: Commit each implementation slice**

For each M65 slice, exact-stage the slice spec, slice plan, code, tests, docs, release fixtures, and status files touched by that slice only.
