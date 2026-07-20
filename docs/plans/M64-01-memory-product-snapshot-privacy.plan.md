# M64-01 Memory Product Snapshot and Privacy Contracts Plan

> **For agentic workers:** Keep this slice contract-only. Do not add CLI commands, Desktop UI, or runtime prompt influence in M64-01.

**Goal:** Add typed, serializable, bounded, redacted memory product snapshot contracts in `ruse-memory`.

**Architecture:** `ruse-memory` owns memory privacy and product summaries. Later slices may expose snapshots through `ruse-ui-bridge` and render them in CLI/TUI/Desktop.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M64-memory-personalization.spec.md`
- Read: `docs/specs/M64-01-memory-product-snapshot-privacy.spec.md`
- Read: `crates/ruse-memory/src/record.rs`
- Read: `crates/ruse-memory/src/proposal.rs`
- Read: `crates/ruse-memory/src/rerank.rs`
- Read: `crates/ruse-memory/src/trace.rs`
- Read: `crates/ruse-memory/src/archive.rs`
- Read: `crates/ruse-memory/src/tombstone.rs`
- Read: `crates/ruse-memory/tests/`

- [x] **Step 1: Review existing contracts**

Identify existing structs and redaction helpers that can feed product summaries without copying raw sensitive memory into UI-facing fields.

### Task 2: Write Focused Tests

**Files:**
- Create: `crates/ruse-memory/tests/memory_product_snapshot.rs`

- [x] **Step 1: Add serialization and privacy tests**

Cover:

- stable snapshot JSON shape
- public memory preview
- sensitive memory preview hidden by default
- proposal status/eligibility summary
- retrieval selected/rejected counts
- influence trace redaction
- tombstone exclusion from active summaries
- bounded diagnostics

- [x] **Step 2: Run focused test and verify failure**

```bash
cargo test -p ruse-memory --test memory_product_snapshot
```

Expected: tests fail before the snapshot contract exists.

### Task 3: Implement Snapshot Contracts

**Files:**
- Create or modify: `crates/ruse-memory/src/product.rs`
- Modify: `crates/ruse-memory/src/lib.rs`

- [x] **Step 1: Add public summary structs**

Implement:

- `MemoryProductSnapshot`
- `MemoryRecordSummary`
- `MemoryProposalSummary`
- `MemoryRetrievalPreview`
- `MemoryInfluenceSummary`
- `MemoryRetentionControlSummary`
- `MemoryPrivacyDiagnostic`

- [x] **Step 2: Add constructors/converters**

Convert from existing memory records, proposal summaries, retrieval results, influence traces, archives/import reports, and tombstones where available.

- [x] **Step 3: Add bounding/redaction helpers**

Use consistent limits and secret-like redaction for previews, source labels, reasons, diagnostics, and influence summaries.

### Task 4: Verify

**Files:**
- Read: changed files

- [x] **Step 1: Run focused checks**

```bash
cargo test -p ruse-memory --test memory_product_snapshot
cargo test -p ruse-memory
cargo clippy -p ruse-memory --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Expected: focused memory checks pass.

### Task 5: Status and Commit

**Files:**
- Modify: `docs/plans/M64-memory-personalization.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Update status docs after focused gates pass**

Mark M64-01 implementation complete only after focused gates pass. Keep M64 overall in progress.

- [x] **Step 2: Commit**

Exact-stage only M64-01 files and required status docs.

Suggested commit message:

```text
feat: add m64 memory product snapshots
```
