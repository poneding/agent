# M12-04 Memory Portability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add versioned memory archive export/import contracts with retention and sensitivity preservation.

**Architecture:** Implement archive contracts in `agent-memory`; keep persistence and UI out of this slice. Use existing memory record types where possible.

**Tech Stack:** Rust 2024 edition, serde, serde_json, existing `agent-memory`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M12-04-memory-portability.spec.md`
- Read: `crates/agent-memory/src/lib.rs`
- Read: `crates/agent-memory/tests/conversation_memory.rs`
- Read: `crates/agent-memory/tests/project_memory.rs`

- [x] **Step 1: Review memory contracts**

Run:

```bash
sed -n '1,220p' crates/agent-memory/src/lib.rs
sed -n '1,240p' docs/specs/M12-04-memory-portability.spec.md
```

Expected: archive contracts stay in `agent-memory`.

### Task 2: Write RED Archive Tests

**Files:**
- Create: `crates/agent-memory/tests/memory_archive.rs`

- [x] **Step 1: Add failing tests**

Add tests:

- `memory_archive_serializes_stable_shape`
- `memory_archive_export_applies_retention_policy`
- `memory_archive_import_rejects_duplicate_ids`
- `memory_archive_import_preserves_scope_and_sensitivity`
- `memory_archive_round_trip_returns_expected_records`

The tests define the public API for `MemoryArchive`, `MemoryArchiveManifest`, `MemoryArchiveRecord`, `MemoryRetentionPolicy`, `MemoryImportOutcome`, `MemoryImportReport`, `MemoryArchiveError`, `export_memory_archive`, and `import_memory_archive`.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-memory memory_archive
```

Expected: tests fail because archive contracts do not exist.

### Task 3: Implement Archive Contracts

**Files:**
- Modify: `crates/agent-memory/src/lib.rs`
- Create: `crates/agent-memory/src/archive.rs`

- [x] **Step 1: Add archive types**

Implement `MemoryArchive`, `MemoryArchiveManifest`, `MemoryArchiveRecord`, `MemoryRetentionPolicy`, `MemoryImportReport`, and `MemoryArchiveError`.

- [x] **Step 2: Add export helper**

Implement deterministic export from memory records into an archive while applying retention policy.

- [x] **Step 3: Add import validation**

Reject unsupported versions, duplicate ids, empty content, invalid scopes, and sensitivity downgrades.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/agent-memory/tests/memory_archive.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-memory memory_archive
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-04-memory-portability.plan.md`

- [x] **Step 1: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m12 memory portability"
```

Expected: commit succeeds.
