# M25 Packaging, Settings, and Updates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prepare `agent` for repeatable local distribution with packaging, platform paths, settings, secrets, migrations, and update flows.

**Architecture:** `agent-cli` exposes the `agent` binary, Desktop packaging uses Tauri, shared platform paths and settings live behind typed runtime contracts, and migrations run before runtime side effects.

**Tech Stack:** Rust 2024 edition, Tauri bundler, OS keychain adapters, platform dirs, schema migrations, release smoke tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M25 Technical Design

**Files:**
- Create: `docs/specs/M25-00-packaging-release-technical-design.spec.md`
- Create: `docs/plans/M25-00-packaging-release-technical-design.plan.md`

- [x] **Step 1: Write the M25 technical design baseline**

Define platform paths, settings profiles, secret backends, migration runner, release artifacts, channels, signing hooks, and update failure behavior.

- [x] **Step 2: Write the M25 design execution plan**

Document CLI and Desktop release slices with artifact smoke tests.

### Task 2: Decompose M25 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M25-01-platform-paths-settings.spec.md`
- Create: `docs/plans/M25-01-platform-paths-settings.plan.md`
- Create: `docs/specs/M25-02-os-keychain-secret-store.spec.md`
- Create: `docs/plans/M25-02-os-keychain-secret-store.plan.md`
- Create: `docs/specs/M25-03-startup-migrations.spec.md`
- Create: `docs/plans/M25-03-startup-migrations.plan.md`
- Create: `docs/specs/M25-04-cli-release-packaging.spec.md`
- Create: `docs/plans/M25-04-cli-release-packaging.plan.md`
- Create: `docs/specs/M25-05-desktop-bundle-updater.spec.md`
- Create: `docs/plans/M25-05-desktop-bundle-updater.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines platform behavior, failure handling, security, and release verification.

- [x] **Step 2: Create slice plans**

Each plan includes smoke-test commands and migration rollback checks.

### Task 3: Implement M25 Slices in Order

**Files:**
- Execute: `docs/plans/M25-01-platform-paths-settings.plan.md`
- Execute: `docs/plans/M25-02-os-keychain-secret-store.plan.md`
- Execute: `docs/plans/M25-03-startup-migrations.plan.md`
- Execute: `docs/plans/M25-04-cli-release-packaging.plan.md`
- Execute: `docs/plans/M25-05-desktop-bundle-updater.plan.md`

- [x] **Step 1: Implement platform paths and settings**

Resolve config, data, cache, log, memory, and plugin directories consistently.

- [x] **Step 2: Implement OS keychain secret store**

Add keychain backends with env fallback and redacted diagnostics.

- [x] **Step 3: Implement startup migrations**

Run idempotent config, storage, memory, and plugin migrations before runtime startup.

- [x] **Step 4: Implement CLI release packaging**

Add release profiles, binary artifact generation, and smoke tests.

- [x] **Step 5: Implement Desktop bundle and updater hooks**

Add Tauri bundle settings, update manifest handling, and local update failure tests.

### Task 4: Complete M25

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M25-packaging-release-settings-updates.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [x] **Step 2: Run release smoke checks**

Run the CLI and Desktop smoke commands defined by the M25 slice plans.

Expected: generated artifacts start, report version, and read settings without raw secrets.

- [x] **Step 3: Mark M25 complete**

Update milestone status and record the next release-readiness focus.

- [x] **Step 4: Commit M25 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m25 packaging release"
```

Expected: commit succeeds.
