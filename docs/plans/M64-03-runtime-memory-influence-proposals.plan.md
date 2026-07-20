# M64-03 Runtime Memory Influence and Proposal Loop Plan

> **For agentic workers:** Keep retrieval, rerank, proposal validation, product snapshots, and redaction in `ruse-memory`. Keep prompt-layer construction in `ruse-context`. `ruse-core` should orchestrate turn-level memory evidence rather than owning memory policy.

**Goal:** Let runtime turns include selected memory through typed prompt layers, expose safe influence evidence, and create reviewable memory write proposals from turn evidence without committing them.

**Architecture:** `ruse-core` accepts a typed runtime memory input, calls `ruse-memory` retrieval/rerank/proposal contracts, asks `ruse-context` to convert selected memories into prompt layers, and returns `RuntimeMemoryTurnEvidence` for CLI/bridge/UI consumers in later slices.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M64-memory-personalization.spec.md`
- Read: `docs/specs/M64-03-runtime-memory-influence-proposals.spec.md`
- Read: `crates/ruse-memory/src/product.rs`
- Read: `crates/ruse-memory/src/proposal.rs`
- Read: `crates/ruse-memory/src/rerank.rs`
- Read: `crates/ruse-context/src/memory_layers.rs`
- Read: `crates/ruse-context/src/prompt.rs`
- Read: `crates/ruse-core/src/turn.rs`
- Read: `crates/ruse-core/src/context.rs`
- Read: `crates/ruse-core/tests/m05_context_memory_integration.rs`

- [x] **Step 1: Review current runtime/context memory baseline**

Confirm `ruse-context` already converts selected memory candidates into privacy-aware prompt layers, while `ruse-core` turn assembly currently uses a minimal `ContextAssembler` and does not yet carry memory influence/proposal evidence.

### Task 2: Write Focused Tests

**Files:**
- Modify: `crates/ruse-context/tests/memory_prompt_layers.rs`
- Create: `crates/ruse-core/tests/runtime_memory_influence.rs`

- [x] **Step 1: Extend context tests**

Prove selected memory layers can be combined with normal prompt layers and preserve priority, visibility, sensitivity, source, and token accounting.

- [x] **Step 2: Add core runtime memory tests**

Cover:

- disabled memory returns no layers and an info diagnostic
- selected memory affects assembled model request messages
- influence evidence reports selected/rejected counts with redacted sensitive previews
- proposal report creates pending summaries from bounded turn evidence
- serialized runtime evidence excludes raw sensitive memory and proposal content

- [x] **Step 3: Run focused tests and verify failure before implementation**

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-core --test runtime_memory_influence
```

Expected: new core tests fail until runtime memory contracts are implemented.

### Task 3: Implement Context Assembly Support

**Files:**
- Modify: `crates/ruse-context/src/memory_layers.rs`
- Modify if needed: `crates/ruse-context/src/lib.rs`

- [x] **Step 1: Add convenience assembly helpers if needed**

Keep existing `memory_prompt_layers` privacy semantics. Add only the smallest helper needed to combine selected memories with other prompt layers in tests/runtime.

- [x] **Step 2: Preserve sensitive memory behavior**

Sensitive memory remains hidden/user-private. Public/internal visibility remains unchanged unless a test exposes a contract gap.

### Task 4: Implement Core Runtime Memory Contracts

**Files:**
- Create or modify: `crates/ruse-core/src/memory.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Modify if needed: `crates/ruse-core/src/context.rs`
- Modify if needed: `crates/ruse-core/src/turn.rs`

- [x] **Step 1: Add public runtime memory structs**

Implement:

- `RuntimeMemoryConfig`
- `RuntimeMemoryInput`
- `RuntimeMemoryInfluenceReport`
- `RuntimeMemoryProposalReport`
- `RuntimeMemoryTurnEvidence`
- `RuntimeMemoryProposalPolicy`

- [x] **Step 2: Add influence assembly**

Retrieve/rerank candidates, convert selected memories into prompt layers, assemble safe influence/product summaries, and degrade cleanly when memory is disabled or empty.

- [x] **Step 3: Add proposal creation**

Create pending `MemoryWriteProposal` values from bounded turn evidence. Return safe proposal summaries only; do not commit records.

### Task 5: Verify and Update Status

**Files:**
- Modify: `docs/plans/M64-memory-personalization.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Run focused gates**

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-context --test memory_prompt_layers
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-core --test runtime_memory_influence
CARGO_TARGET_DIR=target/m64-dev cargo clippy -p ruse-context -p ruse-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 2: Update status docs**

Mark M64-03 complete only after focused gates pass. Keep M64 overall in progress and set M64-04 as the next active slice.

### Task 6: Commit

**Files:**
- Modify/create only M64-03 files and required status docs.

- [x] **Step 1: Commit**

Exact-stage only M64-03 files and required status docs.

Suggested commit message:

```text
feat: add m64 runtime memory influence
```
