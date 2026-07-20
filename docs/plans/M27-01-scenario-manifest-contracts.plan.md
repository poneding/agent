# M27-01 Scenario Manifest Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the `ruse-eval` crate and versioned evaluation scenario manifest contracts.

**Architecture:** `ruse-eval::manifest` owns parsing, validation, and path normalization. Runner, CLI, and CI code consume validated manifests later.

**Tech Stack:** Rust 2024 edition, serde, camino, thiserror or typed validation errors, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M27-01-scenario-manifest-contracts.spec.md`
- Read: `docs/architecture/README.md`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,260p' docs/specs/M27-01-scenario-manifest-contracts.spec.md
```

Expected: the spec defines manifest contracts, validation, privacy, and tests.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-eval/tests/manifest.rs`

- [x] **Step 1: Add manifest serde and validation tests**

Cover a valid manifest, invalid schema version, invalid scenario ID, path traversal, missing fixture requirements, and secret-safe diagnostics.

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
cargo test -p ruse-eval manifest
```

Expected: tests fail before the crate/contracts exist.

### Task 3: Implement Manifest Contracts

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-eval/Cargo.toml`
- Create: `crates/ruse-eval/src/lib.rs`
- Create: `crates/ruse-eval/src/manifest.rs`

- [x] **Step 1: Add `ruse-eval` crate to the workspace**

Expected: crate compiles and uses workspace lint/rust-version conventions.

- [x] **Step 2: Add public manifest structs and validation**

Expected: manifests are serializable, reject unsafe input, and normalize fixture paths.

- [x] **Step 3: Run focused tests and verify success**

Run:

```bash
cargo test -p ruse-eval manifest
```

Expected: focused tests pass.

### Task 4: Update Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M27-runtime-evaluation-harness.plan.md`

- [x] **Step 1: Mark M27-01 complete**

Expected: planning status and active work point to M27-02.

### Task 5: Run Verification and Commit

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [x] **Step 1: Run focused and standard-enough checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-eval --all-targets -- -D warnings
cargo test -p ruse-eval
```

Expected: all focused checks pass.

- [x] **Step 2: Commit M27-01**

Run:

```bash
git add Cargo.toml Cargo.lock crates/ruse-eval docs
git commit -m "feat: add eval scenario manifests"
```

Expected: commit succeeds with source, tests, and documentation together.
