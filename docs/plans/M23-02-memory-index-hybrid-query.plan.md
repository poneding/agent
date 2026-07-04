# M23-02 Memory Index and Hybrid Query Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic memory indexing and hybrid lexical/semantic query in `agent-memory`.

**Architecture:** `agent-memory` owns indexing and candidate scoring. `agent-context` consumes retrieval output later and does not own the index.

**Tech Stack:** Rust 2024 edition, serde, deterministic lexical tokenization, cosine similarity, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M23-02-memory-index-hybrid-query.spec.md`
- Read: `docs/specs/M23-00-memory-retrieval-technical-design.spec.md`
- Read: `docs/specs/M23-01-embedding-provider-contracts.spec.md`
- Read: `crates/agent-memory/src/record.rs`

- [x] **Step 1: Review index scope**

Run:

```bash
sed -n '1,260p' docs/specs/M23-02-memory-index-hybrid-query.spec.md
```

Expected: query validation, filtering, scoring, and deterministic ordering are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-memory/tests/memory_index_hybrid_query.rs`

- [x] **Step 1: Add lexical retrieval tests**

Cover indexing, token matching, scope filtering, sensitivity ceiling, top-k, minimum score, and tie ordering.

- [x] **Step 2: Add semantic fixture tests**

Cover query vectors, cosine similarity, missing embeddings fallback, and hybrid weights.

- [x] **Step 3: Add invalid query tests**

Cover empty query text, empty scopes, zero top-k, invalid weights, and non-finite scores.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-memory memory_index_hybrid_query
```

Expected: tests fail because index contracts are missing.

### Task 3: Implement Hybrid Index

**Files:**
- Create: `crates/agent-memory/src/index.rs`
- Modify: `crates/agent-memory/src/lib.rs`

- [x] **Step 1: Add query and candidate contracts**

Implement `HybridMemoryQuery`, `MemoryRetrievalCandidate`, index stats, and validation helpers.

- [x] **Step 2: Add lexical scoring**

Normalize terms, count matches, compute bounded lexical scores, and preserve matched terms.

- [x] **Step 3: Add semantic scoring**

Store embedding vectors by record id and compute cosine similarity when query embeddings are supplied or fixture semantics are available.

- [x] **Step 4: Add hybrid ordering**

Fuse scores by weights, filter by `minimum_score`, apply `top_k`, and sort deterministically.

### Task 4: Verify

**Files:**
- Read: `crates/agent-memory/tests/memory_index_hybrid_query.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-memory memory_index_hybrid_query
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
- Modify: `docs/plans/M23-02-memory-index-hybrid-query.plan.md`
- Modify: `docs/plans/M23-memory-retrieval-quality.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M23-02 complete**

Advance active implementation focus to M23-03.

- [x] **Step 2: Commit M23-02**

Run:

```bash
git add crates/agent-memory docs Cargo.toml Cargo.lock
git commit -m "feat: add hybrid memory retrieval"
```

Expected: commit succeeds.
