# M23-04 Memory Write Proposals Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add reviewed memory write proposals and safe transition logic before committing new memories.

**Architecture:** `ruse-memory` owns proposal records, stores, validation, transitions, and conversion to `MemoryRecord`. Core/UI integration remains outside this slice unless needed for typed events.

**Tech Stack:** Rust 2024 edition, serde, ruse-types event ids, deterministic state transitions, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M23-04-memory-write-proposals.spec.md`
- Read: `docs/specs/M23-00-memory-retrieval-technical-design.spec.md`
- Read: `crates/ruse-memory/src/store.rs`
- Read: `crates/ruse-memory/src/audit.rs`

- [x] **Step 1: Review proposal scope**

Run:

```bash
sed -n '1,260p' docs/specs/M23-04-memory-write-proposals.spec.md
```

Expected: proposal contracts, transition rules, and privacy behavior are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-memory/tests/memory_write_proposals.rs`

- [x] **Step 1: Add validation tests**

Cover empty ids, empty content, empty summary, invalid confidence, duplicate pending ids, and secret-safe errors.

- [x] **Step 2: Add transition tests**

Cover pending to approved/rejected/expired, approved to committed, duplicate commit, and invalid terminal transitions.

- [x] **Step 3: Add conversion and auto-approval tests**

Cover approved proposal conversion to `MemoryRecord` and advisory auto-approval eligibility metadata.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p ruse-memory memory_write_proposals
```

Expected: tests fail because proposal contracts are missing.

### Task 3: Implement Proposal Store

**Files:**
- Create: `crates/ruse-memory/src/proposal.rs`
- Modify: `crates/ruse-memory/src/lib.rs`
- Modify: `crates/ruse-memory/src/error.rs`

- [x] **Step 1: Add proposal contracts**

Implement proposal ids, statuses, decisions, auto-approval metadata, and proposal records.

- [x] **Step 2: Add in-memory proposal store**

Implement add, list, inspect, resolve, expire, and commit behavior with deterministic ordering.

- [x] **Step 3: Add conversion to memory records**

Create `MemoryRecord` from approved proposal content, source events, confidence, sensitivity, and scope.

- [x] **Step 4: Add redaction and audit summaries**

Ensure transition errors and proposal summaries do not expose raw secret-like content.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-memory/tests/memory_write_proposals.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-memory memory_write_proposals
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `docs/plans/M23-04-memory-write-proposals.plan.md`
- Modify: `docs/plans/M23-memory-retrieval-quality.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M23-04 complete**

Advance active implementation focus to M23-05.

- [x] **Step 2: Commit M23-04**

Run:

```bash
git add crates/ruse-memory docs Cargo.toml Cargo.lock
git commit -m "feat: add memory write proposals"
```

Expected: commit succeeds.
