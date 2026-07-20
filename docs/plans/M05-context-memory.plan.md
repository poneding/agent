# M05 Context and Memory MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add bounded context assembly and scoped memory while preserving reproducible runtime behavior.

**Architecture:** Implement this milestone behind the crate boundaries named in the matching spec. Keep shared contracts in lower-level crates, route orchestration through `ruse-core`, and expose user-facing behavior through CLI or UI bridge crates only when the spec calls for it.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde-compatible contracts where durable or wire formats are involved, Tokio for async runtime paths that need it, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Milestone Inputs

**Files:**
- Read: `docs/specs/M05-context-memory.spec.md`
- Read: `docs/milestones/README.md`
- Read: `docs/PRD.md`

- [ ] **Step 1: Review the milestone spec**

Run:

```bash
sed -n '1,260p' docs/specs/M05-context-memory.spec.md
```

Expected: the spec describes purpose, scope, contracts, tests, and acceptance criteria for M05.

- [ ] **Step 2: Confirm workspace health before changes**

Run:

```bash
cargo test --workspace
```

Expected: existing workspace tests pass before this milestone begins.

### Task 2: Add Contract Tests First

**Files:**
- Create/Modify: `crates/ruse-core/tests/m05_context_memory.rs`

- [ ] **Step 1: Write tests for milestone behavior**

Cover these behaviors:

- prompt layer ordering tests
- budget allocation tests
- pin preservation tests
- summary stability tests
- memory retrieval and deletion tests

Create focused tests with names that describe the contract being proven.

Recommended test names:

- `m05_context_memory_validates_public_contracts`
- `m05_context_memory_rejects_invalid_inputs`
- `m05_context_memory_emits_expected_runtime_events`
- `m05_context_memory_preserves_serialized_shape`
- `m05_context_memory_reports_recoverable_errors`

- [ ] **Step 2: Run the focused tests and verify failure**

Run:

```bash
cargo test --workspace m05_context_memory
```

Expected: tests fail because the milestone contracts are not implemented yet.

### Task 3: Implement Milestone Contracts

**Files:**
- Modify/Create: `crates/ruse-context/src/lib.rs`
- Modify/Create: `crates/ruse-memory/src/lib.rs`
- Modify/Create: `crates/ruse-core/src/lib.rs`
- Modify/Create: `crates/ruse-types/src/lib.rs`

- [ ] **Step 1: Add or extend crate modules**

Implement the public contracts listed in the spec:

- PromptLayer contract
- ContextBudget contract
- PinnedMessage contract
- CompactionSummary contract
- MemoryRecord contract

- [ ] **Step 2: Keep dependency direction clean**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: metadata succeeds and newly touched crates do not introduce reverse dependencies into lower-level crates.

- [ ] **Step 3: Run focused tests and verify success**

Run:

```bash
cargo test --workspace m05_context_memory
```

Expected: focused milestone tests pass.

### Task 4: Add Integration Coverage

**Files:**
- Create/Modify: `crates/ruse-core/tests/m05_context_memory_integration.rs`

- [ ] **Step 1: Add integration assertions for acceptance criteria**

Acceptance criteria to encode:

- Runtime assembles deterministic prompt bundles.
- Long conversations compact while preserving pinned constraints.
- Project memory can be added, retrieved, audited, and deleted.

- [ ] **Step 2: Run integration tests**

Run:

```bash
cargo test --workspace m05_context_memory_integration
```

Expected: integration tests pass and fail deterministically when the milestone behavior is removed.

### Task 5: Run Full Verification

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [ ] **Step 1: Format check**

Run:

```bash
cargo fmt --all -- --check
```

Expected: formatting check passes.

- [ ] **Step 2: Lint check**

Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: clippy passes without warnings.

- [ ] **Step 3: Full test suite**

Run:

```bash
cargo test --workspace
```

Expected: all workspace tests pass.

### Task 6: Update Progress and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [ ] **Step 1: Update active milestone status**

Update `docs/milestones/README.md` so the completed milestone is marked as complete or the active focus advances to the next milestone.

- [ ] **Step 2: Commit the milestone**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m05 context memory"
```

Expected: commit succeeds with source, tests, and documentation staged together.
