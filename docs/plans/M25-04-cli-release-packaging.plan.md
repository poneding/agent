# M25-04 CLI Release Packaging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add repeatable CLI release artifact generation, metadata, checksums, and local smoke tests.

**Architecture:** `xtask` orchestrates packaging; `agent-cli` supplies the `agent` binary and smoke commands.

**Tech Stack:** Rust 2024 edition, release profile, xtask command planning, checksum generation, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M25-04-cli-release-packaging.spec.md`
- Read: `xtask/src/lib.rs`
- Read: `crates/agent-cli/Cargo.toml`
- Read: `README.md`

- [ ] **Step 1: Review packaging scope**

Expected: release artifact naming, smoke checks, and output directory rules are clear.

### Task 2: Write Failing Tests

**Files:**
- Modify: `xtask/tests/xtask_plan.rs`
- Create if needed: `xtask/tests/package_cli.rs`

- [ ] **Step 1: Add xtask command tests**

Cover `package-cli`, release channel, target triple, and output directory arguments.

- [ ] **Step 2: Add artifact metadata tests**

Cover deterministic manifest shape and checksum fields.

- [ ] **Step 3: Verify RED**

```bash
cargo test -p xtask package_cli
```

### Task 3: Implement CLI Packaging

**Files:**
- Modify: `xtask/src/lib.rs`
- Modify: `xtask/src/main.rs`
- Modify if needed: `Cargo.toml`
- Modify if needed: `README.md`

- [ ] **Step 1: Add xtask packaging command**

Plan release build, artifact copy, checksum, manifest, and smoke commands.

- [ ] **Step 2: Add smoke checks**

Run artifact version, config path diagnostics, settings load, and fake-provider prompt in temp roots.

- [ ] **Step 3: Add release checklist docs**

Document local CLI packaging and verification commands.

### Task 4: Verify

- [ ] **Step 1: Run focused tests**

```bash
cargo test -p xtask package_cli
cargo xtask package-cli --channel dev
```

- [ ] **Step 2: Run workspace gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M25-04-cli-release-packaging.plan.md`
- Modify: `docs/plans/M25-packaging-release-settings-updates.plan.md`

- [ ] **Step 1: Mark M25-04 complete**

Advance active focus to M25-05.

- [ ] **Step 2: Commit M25-04**

```bash
git add crates xtask docs README.md Cargo.toml Cargo.lock
git commit -m "feat: add cli release packaging"
```
