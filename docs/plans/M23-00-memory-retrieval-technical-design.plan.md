# M23-00 Memory Retrieval Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M23 architecture baseline for hybrid memory retrieval and memory influence tracing.

**Architecture:** This is a documentation-only slice. It defines `agent-memory` ownership for embeddings, indexing, hybrid query, reranking, proposals, tombstones, and privacy boundaries.

**Tech Stack:** Markdown docs, existing memory/context/storage architecture docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M23-memory-retrieval-quality.spec.md`
- Read: `docs/specs/M05-context-memory.spec.md`
- Read: `docs/specs/M12-04-memory-portability.spec.md`
- Read: `docs/architecture/crate-boundaries.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M23-memory-retrieval-quality.spec.md
```

Expected: milestone scope defines embeddings, hybrid retrieval, reranking, write proposals, and privacy controls.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M23-00-memory-retrieval-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document embedding provider, memory index, retrieval flow, write proposals, deletion tombstones, security, and testing strategy.

- [ ] **Step 2: Self-review design**

Check that sensitive memory and embeddings are handled as private data and deleted records cannot reappear.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M23-00-memory-retrieval-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M23-00-memory-retrieval-technical-design.spec.md`
- Add: `docs/plans/M23-00-memory-retrieval-technical-design.plan.md`
- Modify: `docs/plans/M23-memory-retrieval-quality.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M23-00-memory-retrieval-technical-design.spec.md docs/plans/M23-00-memory-retrieval-technical-design.plan.md docs/plans/M23-memory-retrieval-quality.plan.md
git commit -m "docs: add m23 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
