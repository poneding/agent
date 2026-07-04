# M25-03 Startup Migrations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a startup migration runner that plans, applies, verifies, and audits local-state upgrades before runtime side effects.

**Architecture:** `agent-core` orchestrates; storage, memory, plugin, and prompt crates expose migration participants.

**Tech Stack:** Rust 2024 edition, serde, schema version records, idempotent migration fixtures, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M25-03-startup-migrations.spec.md`
- Read: `docs/architecture/storage.md`
- Read: `crates/agent-storage/src/migration.rs`
- Read: `crates/agent-storage/tests/schema_migration.rs`

- [x] **Step 1: Review migration boundaries**

Expected: existing schema migration code can be reused behind startup orchestration.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-core/tests/startup_migrations.rs`
- Modify if needed: `crates/agent-storage/tests/schema_migration.rs`

- [x] **Step 1: Add runner plan/apply tests**

Cover ordered steps, no-op repeat, and audit records.

- [x] **Step 2: Add failure isolation tests**

Prove runtime startup does not proceed after migration failure.

- [x] **Step 3: Verify RED**

```bash
cargo test -p agent-core startup_migrations
```

### Task 3: Implement Startup Migration Runner

**Files:**
- Modify: `crates/agent-core/src/lib.rs`
- Create if needed: `crates/agent-core/src/startup_migration.rs`
- Modify if needed: `crates/agent-storage/src/lib.rs`
- Modify if needed: `crates/agent-memory/src/lib.rs`
- Modify if needed: `crates/agent-plugin/src/lib.rs`

- [x] **Step 1: Add migration contracts**

Define plan, step, participant, result, and diagnostics.

- [x] **Step 2: Wire domain participants**

Use existing storage migration behavior and add no-op participants where domain migrations are not yet required.

- [x] **Step 3: Block runtime on failure**

Return recoverable startup diagnostics before any agent turn or background task starts.

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
cargo test -p agent-core startup_migrations
cargo test -p agent-storage schema_migration
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
- Modify: `docs/plans/M25-03-startup-migrations.plan.md`
- Modify: `docs/plans/M25-packaging-release-settings-updates.plan.md`

- [x] **Step 1: Mark M25-03 complete**

Advance active focus to M25-04.

- [x] **Step 2: Commit M25-03**

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: add startup migrations"
```
