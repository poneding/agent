# M23-01 Embedding Provider Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add async embedding provider contracts and deterministic fixture embeddings in `agent-memory`.

**Architecture:** `agent-memory` owns embedding types, validation, and fixture providers. No core, UI, HTTP, or concrete model-provider dependency belongs in this slice.

**Tech Stack:** Rust 2024 edition, async-trait, serde, deterministic hashing, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M23-01-embedding-provider-contracts.spec.md`
- Read: `docs/specs/M23-00-memory-retrieval-technical-design.spec.md`
- Read: `crates/agent-memory/src/record.rs`
- Read: `crates/agent-memory/src/error.rs`

- [x] **Step 1: Review embedding scope**

Run:

```bash
sed -n '1,260p' docs/specs/M23-01-embedding-provider-contracts.spec.md
```

Expected: trait ownership, validation rules, and fixture behavior are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-memory/tests/embedding_provider.rs`

- [x] **Step 1: Add fixture determinism tests**

Cover repeated requests, stable vector dimensions, stable content hashes, and input-order preservation.

- [x] **Step 2: Add validation tests**

Cover empty provider/model/input fields, duplicate record ids, expected dimension mismatch, and non-finite vector rejection.

- [x] **Step 3: Add redaction tests**

Ensure errors do not include raw `sk-`, `token`, `password`, `api_key`, or memory text.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-memory embedding_provider
```

Expected: tests fail because embedding contracts are missing.

### Task 3: Implement Embedding Contracts

**Files:**
- Create: `crates/agent-memory/src/embedding.rs`
- Modify: `crates/agent-memory/src/lib.rs`
- Modify: `crates/agent-memory/src/error.rs`
- Modify if needed: `Cargo.toml`

- [x] **Step 1: Add public types and trait**

Implement `EmbeddingProvider`, `EmbeddingRequest`, `EmbeddingInput`, `EmbeddingBatch`, `EmbeddingVector`, and `EmbeddingMetadata`.

- [x] **Step 2: Add validation**

Reject empty fields, duplicate ids, empty batches, mismatched dimensions, and non-finite vector values with secret-safe errors.

- [x] **Step 3: Add deterministic fixture provider**

Generate stable vectors from record id and text hash without network, env, time, or randomness.

- [x] **Step 4: Export contracts**

Re-export embedding contracts from `agent-memory`.

### Task 4: Verify

**Files:**
- Read: `crates/agent-memory/tests/embedding_provider.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-memory embedding_provider
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
- Modify: `docs/plans/M23-01-embedding-provider-contracts.plan.md`
- Modify: `docs/plans/M23-memory-retrieval-quality.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M23-01 complete**

Advance active implementation focus to M23-02.

- [x] **Step 2: Commit M23-01**

Run:

```bash
git add crates/agent-memory docs Cargo.toml Cargo.lock
git commit -m "feat: add memory embedding contracts"
```

Expected: commit succeeds.
