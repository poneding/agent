# M65-01 Release Baseline and Beta Gate Contracts Plan

> **For agentic workers:** This slice is planning-first. Do not implement package commands unless the plan is explicitly expanded. The outcome should make M65-02 through M65-06 executable.

**Goal:** Document the beta release baseline, artifact matrix, beta gate contract shape, support evidence inventory, and gap-to-slice mapping.

**Architecture:** `xtask` remains the release gate orchestrator; runtime crates own typed diagnostics; CLI/Desktop render bounded summaries.

**Tech Stack:** Markdown docs, Rust 2024 edition for future contracts, existing `xtask` package commands, Cargo fmt/clippy/test for future implementation slices.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M65-packaging-release-beta-gate.spec.md`
- Read: `docs/specs/M65-01-release-baseline-beta-gate.spec.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `docs/specs/M25-packaging-release-settings-updates.spec.md`
- Read: `docs/specs/M29-ci-quality-evidence-release-gates.spec.md`
- Read: `xtask/src/lib.rs`
- Read: `xtask/tests/package_cli.rs`
- Read: `xtask/tests/package_desktop.rs`

- [x] **Step 1: Review release baseline**

Expected: current package CLI/Desktop commands, eval gates, migration foundations, update manifest contracts, and release smoke gaps are understood.

### Task 2: Document Release Baseline

**Files:**
- Create if needed: `docs/release/beta-gate.md`

- [x] **Step 1: Add current capability baseline**

Document existing release capabilities:

- `cargo xtask package-cli --channel dev`
- `cargo xtask package-desktop --channel dev`
- `cargo xtask eval`
- Desktop tests/build
- startup migration tests
- update manifest validation tests

- [x] **Step 2: Add beta artifact matrix**

Document CLI package, Desktop bundle/plan, update manifest, eval report, migration diagnostics, provider diagnostics, and support bundle evidence.

- [x] **Step 3: Add gate status definitions**

Define pass/fail/degraded/not-run semantics for beta gate checks.

### Task 3: Document Support Bundle Inventory

**Files:**
- Create if needed: `docs/release/support-bundle.md`

- [x] **Step 1: Add support bundle sections**

Document allowlisted sections:

- app/runtime versions
- platform paths
- config summary
- provider readiness without secret values
- startup migration report
- recent redacted diagnostics
- eval/package artifact paths
- Desktop update manifest validation
- extension health summary when available

- [x] **Step 2: Add privacy rules**

State forbidden content: raw secrets, raw prompts, tool input JSON, sensitive memory, embeddings, unbounded logs, and environment dumps.

### Task 4: Update Milestone Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M65-packaging-release-beta-gate.plan.md`

- [x] **Step 1: Register M65 and M65-01**

Add M65 milestone spec/plan and M65 slice index entries. Mark M65-01 planning complete only after docs and checks pass.

- [x] **Step 2: Keep future slices pending**

Do not mark implementation complete for M65-02 through M65-06.

### Task 5: Verify

**Files:**
- All M65 planning docs.

- [x] **Step 1: Run docs gate**

```bash
git diff --check
```

Expected: no whitespace errors.

- [x] **Step 2: Optional command sanity checks**

Run only if useful:

```bash
cargo xtask --help
cargo xtask eval
```

Expected: command paths referenced by planning docs still exist.

### Task 6: Commit

**Files:**
- M65 planning docs and milestone/roadmap status files only.

- [x] **Step 1: Commit M65 planning**

Suggested commit message:

```text
docs: add m65 packaging beta gate plan
```
