# M64-04 TUI/Desktop Memory Surfaces Plan

> **For agentic workers:** Keep memory policy, privacy, retention, retrieval, proposal eligibility, and redaction in `ruse-memory`/`ruse-core`. `ruse-ui-bridge`, TUI, and Desktop should expose already-safe summaries and must not recalculate memory decisions.

**Goal:** Render memory records, proposals, retrieval influence, diagnostics, privacy/retention state, and disabled-memory states in TUI and Desktop from one shared bridge snapshot.

**Architecture:** `ruse-memory` produces product snapshots. `ruse-core` produces runtime memory evidence. `ruse-ui-bridge` maps those contracts into UI-safe frames. TUI and Desktop reduce `memory_snapshot` frames into presentation state and render bounded summaries only.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M64-memory-personalization.spec.md`
- Read: `docs/specs/M64-04-tui-desktop-memory-surfaces.spec.md`
- Read: `crates/ruse-memory/src/product.rs`
- Read: `crates/ruse-core/src/memory.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-tui/src/task_panel.rs`
- Read: `crates/ruse-tui/tests/task_panel.rs`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src/state/conversation.ts`
- Read: `apps/desktop/src/components/`
- Read: `apps/desktop/src/__tests__/`

- [x] **Step 1: Review current product and runtime memory contracts**

Confirm M64-01 product snapshots and M64-03 runtime evidence already provide bounded/redacted memory summaries, retrieval/influence counts, proposal summaries, retention controls, tombstone counts, and diagnostics.

- [x] **Step 2: Review existing UI surface patterns**

Confirm the current bridge/TUI/Desktop patterns for M63 team workflow snapshots, task panels, reducer state, and fixture-backed UI tests. Reuse those patterns instead of starting broad Desktop productization work.

### Task 2: Add Bridge Memory Snapshot Frame

**Files:**
- Modify: `crates/ruse-ui-bridge/Cargo.toml`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Create: `crates/ruse-ui-bridge/tests/memory_snapshot.rs`

- [x] **Step 1: Define UI-safe memory structs**

Add bridge contracts such as:

- `UiMemorySnapshot`
- `UiMemoryRecordSummary`
- `UiMemoryProposalSummary`
- `UiMemoryRetrievalCandidateSummary`
- `UiMemoryInfluenceSummary`
- `UiMemoryDiagnostic`
- `UiMemoryRetentionSummary`

Use Eq-friendly fields and display-ready values only. Do not include raw memory text, raw proposal content, embeddings, raw prompts, or unbounded diagnostic text.

- [x] **Step 2: Add JSON frame variant**

Add a `memory_snapshot` `UiJsonFrame` variant that serializes/deserializes deterministically and can be ignored safely by older reducers.

- [x] **Step 3: Add conversion helpers**

Map from `MemoryProductSnapshot` and, where needed, `RuntimeMemoryTurnEvidence` into bridge structs without changing memory policy semantics.

- [x] **Step 4: Test bridge safety**

Tests must prove:

- stable JSON frame shape
- active record/proposal/retrieval/influence/diagnostic counts survive conversion
- sensitive previews and proposal content remain redacted
- empty and disabled snapshots serialize cleanly
- tombstone/retention state is visible as counts or summaries, not raw deleted content

### Task 3: Render TUI Memory State

**Files:**
- Modify: `crates/ruse-tui/src/task_panel.rs`
- Modify: `crates/ruse-tui/tests/task_panel.rs`

- [x] **Step 1: Extend panel state**

Add memory snapshot state to the existing task/status panel model. Follow M63 team workflow rendering style: consume bridge frame data and keep local logic limited to layout and labels.

- [x] **Step 2: Render memory summary**

Render:

- memory enabled/disabled or degraded state
- active record count and representative redacted previews
- pending/reviewable proposal count
- selected/rejected retrieval influence counts
- diagnostics
- retention and tombstone counts

- [x] **Step 3: Test TUI output**

Tests should cover normal, empty, disabled/degraded, and redacted-sensitive snapshots.

### Task 4: Render Desktop Memory Surface

**Files:**
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src/state/conversation.ts`
- Create or modify: `apps/desktop/src/components/MemoryView.tsx`
- Modify existing app route/view wiring as needed
- Create: `apps/desktop/src/__tests__/memory-ui.test.tsx`

- [x] **Step 1: Add TypeScript frame types**

Mirror the bridge `memory_snapshot` frame shape with typed, fixture-friendly interfaces. Keep naming and field casing aligned with serialized Rust frames.

- [x] **Step 2: Reduce memory frames**

Store the latest memory snapshot in existing Desktop conversation/app state. Unknown or missing memory frames should leave the UI stable.

- [x] **Step 3: Add minimal product surface**

Add a focused Desktop memory view/card/section using the existing visual system. It should display:

- memory readiness and privacy state
- active records
- proposals
- retrieval/influence counts
- diagnostics
- retention/tombstone counts

Do not add proposal action buttons in this slice unless an existing typed command is already available through the bridge and tested.

- [x] **Step 4: Test Desktop rendering**

Vitest coverage should prove:

- reducer stores `memory_snapshot`
- component renders records/proposals/influence/diagnostics
- disabled/empty states are clear
- raw secret-like text and raw proposal content are absent

### Task 5: Verify and Update Status

**Files:**
- Modify: `docs/plans/M64-memory-personalization.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Run focused gates**

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-ui-bridge --test memory_snapshot
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-tui --test task_panel
npm --prefix apps/desktop test -- --run src/__tests__/memory-ui.test.tsx
npm --prefix apps/desktop run build
cargo fmt --all -- --check
git diff --check
```

Completed evidence:

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-ui-bridge --test memory_snapshot
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-tui --test task_panel
CARGO_TARGET_DIR=target/m64-dev cargo clippy -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings
npm --prefix apps/desktop test -- --run src/__tests__/memory-ui.test.tsx
npm --prefix apps/desktop run build
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 2: Update status docs**

Mark M64-04 complete only after focused gates pass. Keep M64 overall in progress and set M64-05 Memory Smoke, Docs, and Closeout as the next active slice.

### Task 6: Commit

**Files:**
- Modify/create only M64-04 files and required status docs.

- [x] **Step 1: Commit planning**

Exact-stage the M64-04 spec/plan and status docs.

Suggested commit message:

```text
docs: add m64 memory surfaces plan
```

- [x] **Step 2: Commit implementation**

Exact-stage bridge/TUI/Desktop implementation, tests, and status docs.

Suggested commit message:

```text
feat: add m64 memory ui surfaces
```
