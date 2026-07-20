# M23-03 Memory Rerank and Influence Trace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add reranking, selected/rejected retrieval results, influence traces, and prompt-layer conversion for retrieved memories.

**Architecture:** `ruse-memory` owns rerank and trace logic. `ruse-context` owns conversion into prompt layers. `ruse-core` only emits trace evidence when integrating retrieval into turns.

**Tech Stack:** Rust 2024 edition, serde, deterministic fixture reranker, prompt-layer contracts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M23-03-memory-rerank-influence-trace.spec.md`
- Read: `docs/specs/M23-02-memory-index-hybrid-query.spec.md`
- Read: `crates/ruse-context/src/prompt.rs`
- Read: `crates/ruse-memory/src/record.rs`

- [x] **Step 1: Review rerank and trace scope**

Run:

```bash
sed -n '1,280p' docs/specs/M23-03-memory-rerank-influence-trace.spec.md
```

Expected: selected/rejected result shape, trace privacy, and context adapter boundaries are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-memory/tests/memory_rerank_influence_trace.rs`
- Create: `crates/ruse-context/tests/memory_prompt_layers.rs`

- [x] **Step 1: Add reranker result tests**

Cover deterministic selection, rejected reasons, duplicate handling, max-selected, and empty candidate behavior.

- [x] **Step 2: Add trace redaction tests**

Cover sensitive text omission, query hash stability, score summaries, and serialized trace shape.

- [x] **Step 3: Add context adapter tests**

Cover selected memory conversion into memory prompt layers with source ids, sensitivity mapping, and token estimates.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p ruse-memory memory_rerank_influence_trace
cargo test -p ruse-context memory_prompt_layers
```

Expected: tests fail because rerank, trace, and adapter contracts are missing.

### Task 3: Implement Rerank and Trace Contracts

**Files:**
- Create: `crates/ruse-memory/src/rerank.rs`
- Create: `crates/ruse-memory/src/trace.rs`
- Modify: `crates/ruse-memory/src/lib.rs`
- Create: `crates/ruse-context/src/memory_layers.rs`
- Modify: `crates/ruse-context/src/lib.rs`

- [x] **Step 1: Add reranker contracts**

Implement request/result structs, selected/rejected candidates, reason codes, and fixture reranker.

- [x] **Step 2: Add influence traces**

Implement trace entries with query hash, record ids, score summaries, reasons, scope, sensitivity, and redacted previews.

- [x] **Step 3: Add prompt-layer adapter**

Convert selected memories into `PromptLayer` values using `PromptLayerKind::Memory` and safe source metadata.

- [x] **Step 4: Add validation and redaction**

Reject invalid scores and ensure sensitive text never appears in traces or adapter diagnostics.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-memory/tests/memory_rerank_influence_trace.rs`
- Read: `crates/ruse-context/tests/memory_prompt_layers.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-memory memory_rerank_influence_trace
cargo test -p ruse-context memory_prompt_layers
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
- Modify: `docs/plans/M23-03-memory-rerank-influence-trace.plan.md`
- Modify: `docs/plans/M23-memory-retrieval-quality.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M23-03 complete**

Advance active implementation focus to M23-04.

- [x] **Step 2: Commit M23-03**

Run:

```bash
git add crates/ruse-memory crates/ruse-context docs Cargo.toml Cargo.lock
git commit -m "feat: add memory rerank traces"
```

Expected: commit succeeds.
