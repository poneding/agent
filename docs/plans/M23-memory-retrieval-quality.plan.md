# M23 Memory Retrieval Quality Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add hybrid memory retrieval, embeddings, reranking hooks, retention controls, and auditable memory influence traces.

**Architecture:** `agent-memory` owns indexing, embeddings metadata, retrieval, retention, and memory write proposals. `agent-context` consumes retrieval results as prompt layers. `agent-core` records memory events.

**Tech Stack:** Rust 2024 edition, SQLite-first memory storage, optional vector index adapter, deterministic embedding fixtures, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M23 Technical Design

**Files:**
- Create: `docs/specs/M23-00-memory-retrieval-technical-design.spec.md`
- Create: `docs/plans/M23-00-memory-retrieval-technical-design.plan.md`

- [x] **Step 1: Write the M23 technical design baseline**

Define embedding abstraction, local index shape, hybrid retrieval, reranking, write proposals, deletion tombstones, and privacy traces.

- [x] **Step 2: Write the M23 design execution plan**

Document slice order and deterministic fixture strategy.

### Task 2: Decompose M23 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M23-01-embedding-provider-contracts.spec.md`
- Create: `docs/plans/M23-01-embedding-provider-contracts.plan.md`
- Create: `docs/specs/M23-02-memory-index-hybrid-query.spec.md`
- Create: `docs/plans/M23-02-memory-index-hybrid-query.plan.md`
- Create: `docs/specs/M23-03-memory-rerank-influence-trace.spec.md`
- Create: `docs/plans/M23-03-memory-rerank-influence-trace.plan.md`
- Create: `docs/specs/M23-04-memory-write-proposals.spec.md`
- Create: `docs/plans/M23-04-memory-write-proposals.plan.md`
- Create: `docs/specs/M23-05-memory-editor-portability.spec.md`
- Create: `docs/plans/M23-05-memory-editor-portability.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines retrieval behavior, privacy controls, and tests.

- [x] **Step 2: Create slice plans**

Each plan starts with deterministic retrieval tests before implementation.

### Task 3: Implement M23 Slices in Order

**Files:**
- Execute: `docs/plans/M23-01-embedding-provider-contracts.plan.md`
- Execute: `docs/plans/M23-02-memory-index-hybrid-query.plan.md`
- Execute: `docs/plans/M23-03-memory-rerank-influence-trace.plan.md`
- Execute: `docs/plans/M23-04-memory-write-proposals.plan.md`
- Execute: `docs/plans/M23-05-memory-editor-portability.plan.md`

- [x] **Step 1: Implement embedding provider contracts**

Add embedding provider traits, vector metadata, and deterministic fixtures.

- [x] **Step 2: Implement hybrid memory index**

Combine lexical and semantic retrieval with scope and sensitivity filters.

- [x] **Step 3: Implement rerank and influence traces**

Return selected memories, scores, reasons, and safe trace summaries.

- [x] **Step 4: Implement memory write proposals**

Separate proposed memories from committed records with approval policy.

- [x] **Step 5: Implement editor and portability updates**

Expose list, inspect, delete, export, import, and tombstone behavior.

### Task 4: Complete M23

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M23-memory-retrieval-quality.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [x] **Step 2: Mark M23 complete**

Update milestone status and advance active focus to M24.

- [x] **Step 3: Commit M23 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m23 memory retrieval"
```

Expected: commit succeeds.
