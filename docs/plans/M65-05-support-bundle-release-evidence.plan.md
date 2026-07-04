# M65-05 Support Bundle and Release Evidence Plan

> **For agentic workers:** Support bundles are allowlisted evidence, not raw data archives. Do not collect raw logs, raw environment variables, keychain values, raw prompts, or workspace files.

**Goal:** Add a typed `xtask` support bundle command that writes redacted, bounded beta troubleshooting evidence.

**Architecture:** `xtask` discovers existing release/eval/quality artifacts, records references and checksums, and writes manifest/summary files.

**Tech Stack:** Rust 2024 edition, serde JSON contracts, existing `xtask` release artifacts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M65-05-support-bundle-release-evidence.spec.md`
- Read: `docs/release/support-bundle.md`
- Read: `xtask/src/lib.rs`
- Read: `xtask/src/main.rs`
- Read: `xtask/tests/package_cli.rs`
- Read: `xtask/tests/package_desktop.rs`
- Read: `xtask/tests/quality_gate.rs`

- [x] **Step 1: Review available release evidence**

Existing evidence includes CLI release manifests, Desktop update manifests, Desktop release plans, quality gate reports, and eval reports.

### Task 2: Add Support Bundle Contracts

**Files:**
- Modify: `xtask/src/lib.rs`

- [x] **Step 1: Add options and paths**

Add `SupportBundleOptions`, default output directory, manifest path, and summary path.

- [x] **Step 2: Add typed manifest and summary**

Add `SupportBundleManifest`, `SupportBundleSection`, `SupportBundleSectionStatus`, and `SupportBundleSummary`.

- [x] **Step 3: Add redaction diagnostics**

Record stable diagnostics for omitted raw secrets, raw environments, raw prompts, and unbounded logs.

### Task 3: Implement Bundle Generation

**Files:**
- Modify: `xtask/src/lib.rs`
- Modify: `xtask/src/main.rs`

- [x] **Step 1: Discover allowlisted artifacts**

Discover known release, Desktop, quality gate, and eval artifact references without reading raw logs or arbitrary files.

- [x] **Step 2: Write manifest and summary**

Write `support-bundle-manifest.json` and `support-bundle-summary.txt`.

- [x] **Step 3: Add CLI command**

Add `cargo xtask support-bundle --channel <dev|beta|stable> --out-dir <path>`.

### Task 4: Add Tests

**Files:**
- Create: `xtask/tests/support_bundle.rs`
- Modify: `xtask/tests/xtask_plan.rs`

- [x] **Step 1: Test stable paths**

Assert manifest/summary paths and input artifact directories.

- [x] **Step 2: Test artifact discovery and section statuses**

Create deterministic fixture artifacts and assert included/omitted statuses.

- [x] **Step 3: Test command parsing**

Cover defaults, channel/out-dir options, and invalid args.

- [x] **Step 4: Test manifest writing and summary rendering**

Assert output files and summary counts.

### Task 5: Update Docs And Status

**Files:**
- Modify: `docs/release/support-bundle.md`
- Modify: `docs/release/beta-gate.md`
- Modify: `docs/plans/M65-packaging-release-beta-gate.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Document command and output**

Document support bundle manifest/summary outputs and forbidden content.

- [x] **Step 2: Update status**

Mark M65-05 complete after focused gates pass and set M65-06 as next active slice.

### Task 6: Verify

- [x] **Step 1: Run focused tests**

```bash
CARGO_TARGET_DIR=target/m65-dev cargo test -p xtask --test support_bundle
CARGO_TARGET_DIR=target/m65-dev cargo test -p xtask --test xtask_plan
```

- [x] **Step 2: Run final focused gates**

```bash
CARGO_TARGET_DIR=target/m65-dev cargo clippy -p xtask --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 3: Run optional local smoke**

```bash
cargo xtask support-bundle --channel dev
```

### Task 7: Commit

**Files:**
- M65-05 spec/plan, `xtask` support bundle code/tests, release docs, milestone/roadmap status.

- [ ] **Step 1: Commit implementation**

Suggested commit message:

```text
feat: add m65 support bundle evidence
```
