# M65-02 CLI Package and Install Smoke Plan

> **For agentic workers:** Extend the existing `xtask package-cli` path. Do not move release logic into `agent-cli`; the packaged binary only supplies smoke command behavior.

**Goal:** Add beta-grade CLI package install smoke evidence with isolated roots, stable manifest fields, focused tests, and release docs.

**Architecture:** `xtask` plans/builds/copies/checksums/smokes the CLI artifact; `agent-cli` and runtime crates provide command behavior and diagnostics.

**Tech Stack:** Rust 2024 edition, existing `xtask` release structs, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M65-02-cli-package-install-smoke.spec.md`
- Read: `docs/specs/M25-04-cli-release-packaging.spec.md`
- Read: `xtask/src/lib.rs`
- Read: `xtask/tests/package_cli.rs`
- Read: `xtask/tests/xtask_plan.rs`
- Read: `docs/release/beta-gate.md`

- [x] **Step 1: Review existing CLI packaging path**

Expected: M25-04 already builds, copies, checksums, manifests, and smoke-tests the CLI artifact.

### Task 2: Add Failing Tests

**Files:**
- Modify: `xtask/tests/package_cli.rs`

- [x] **Step 1: Assert install smoke summary in manifest**

Add assertions that serialized `CliReleaseManifest` includes isolated smoke root, config path, and stable check ids when constructed with install smoke evidence.

- [x] **Step 2: Assert package plan exposes required smoke commands**

Cover version, config check, startup check, JSON prompt smoke, and provider diagnostic smoke if added in this slice.

- [x] **Step 3: Run focused tests**

```bash
CARGO_TARGET_DIR=target/m65-dev cargo test -p xtask package_cli
```

Expected: install-smoke assertions cover the new manifest evidence.

### Task 3: Implement CLI Install Smoke Evidence

**Files:**
- Modify: `xtask/src/lib.rs`
- Modify if needed: `xtask/src/main.rs`

- [x] **Step 1: Add install smoke summary contracts**

Add additive manifest/report fields for smoke root, config path, and smoke check metadata. Preserve existing release manifest fields.

- [x] **Step 2: Populate summary from package plan**

When `run_package_cli` succeeds, write manifest evidence that identifies required smoke checks and isolated roots without raw output.

- [x] **Step 3: Keep privacy boundaries**

Do not store environment dumps, raw command output, raw config contents, or secret values in release evidence.

### Task 4: Update Docs And Status

**Files:**
- Modify: `docs/release/beta-gate.md`
- Create or modify: `docs/cli/release-packaging.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M65-packaging-release-beta-gate.plan.md`
- Modify: `docs/plans/M65-02-cli-package-install-smoke.plan.md`

- [x] **Step 1: Document CLI package smoke**

Explain command, artifacts, isolated roots, manifest fields, and privacy guarantees.

- [x] **Step 2: Update M65 status**

Mark M65-02 complete only after focused tests and docs pass. Set M65-03 as next active slice.

### Task 5: Verify

- [x] **Step 1: Run focused gates**

```bash
CARGO_TARGET_DIR=target/m65-dev cargo test -p xtask package_cli
CARGO_TARGET_DIR=target/m65-dev cargo clippy -p xtask --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 2: Optional package smoke**

Run when local time permits:

```bash
cargo xtask package-cli --channel dev
```

Expected: package artifact, checksum, manifest, and install smoke evidence are written under `target/agent-release/`.

### Task 6: Commit

**Files:**
- M65-02 spec/plan, `xtask` code/tests, release docs, milestone/roadmap status.

- [x] **Step 1: Commit implementation**

Suggested commit message:

```text
feat: add m65 cli package install smoke
```
