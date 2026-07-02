# M64 Memory and Personalization Implementation Plan

> **For agentic workers:** Keep memory product logic in `agent-memory`, `agent-context`, and `agent-core`. CLI/TUI/Desktop must render typed bridge/runtime contracts and must not own privacy, retrieval, proposal, or retention rules.

**Goal:** Make memory useful, reviewable, privacy-safe, and visible across CLI/TUI/Desktop with deterministic smoke evidence.

**Architecture:** `agent-memory` owns records, proposals, archive/import/export, tombstones, retrieval, reranking, and influence traces. `agent-context` assembles memory into prompt layers. `agent-core` orchestrates turn-level retrieval/proposal evidence. `agent-ui-bridge` exposes UI-safe frames. CLI/TUI/Desktop render and command over shared contracts.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `agent-memory`, `agent-context`, `agent-core`, `agent-ui-bridge`, CLI/JSON commands, Ratatui and Desktop fixtures, Cargo fmt/clippy/test, Vitest, Vite build, and `cargo xtask eval`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M64-memory-personalization.spec.md`
- Read: `docs/PRD.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/specs/M05-context-memory.spec.md`
- Read: `docs/specs/M12-04-memory-portability.spec.md`
- Read: `docs/specs/M23-memory-retrieval-quality.spec.md`
- Read: `crates/agent-memory/`
- Read: `crates/agent-context/`
- Read: `crates/agent-core/`
- Read: `crates/agent-ui-bridge/`
- Read: `crates/agent-cli/tests/memory_commands.rs`
- Read: `apps/desktop/src/components/SettingsView.tsx`

- [x] **Step 1: Review existing memory foundations**

Confirm the repo already has memory records, scopes, sensitivity, archives, proposals, hybrid retrieval, reranking, tombstones, editor contracts, influence traces, CLI memory commands, and Desktop settings toggles. M64 should productize these foundations rather than recreate them.

### Task 2: M64-01 Memory Product Snapshot and Privacy Contracts

**Files:**
- Create: `docs/specs/M64-01-memory-product-snapshot-privacy.spec.md`
- Create: `docs/plans/M64-01-memory-product-snapshot-privacy.plan.md`
- Modify: `crates/agent-memory/`
- Modify if needed: `crates/agent-core/`
- Modify if needed: `crates/agent-ui-bridge/`

- [x] **Step 1: Plan M64-01**

Define UI-safe product snapshots for memory records, proposal summaries, retrieval previews, influence traces, retention controls, tombstones, and privacy diagnostics.

- [x] **Step 2: Implement M64-01**

Add typed, serializable, bounded, redacted contracts and focused tests. Keep side effects out of M64-01; this slice should make memory state inspectable before broader CLI/UI/runtime behavior is added.

Expected checks:

```bash
cargo test -p agent-memory
cargo clippy -p agent-memory --all-targets -- -D warnings
```

Completed evidence:

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-memory --test memory_product_snapshot
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-memory
CARGO_TARGET_DIR=target/m64-dev cargo clippy -p agent-memory --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

### Task 3: M64-02 CLI and JSON Memory Review Commands

**Files:**
- Create: `docs/specs/M64-02-cli-json-memory-review.spec.md`
- Create: `docs/plans/M64-02-cli-json-memory-review.plan.md`
- Modify: `crates/agent-cli/`
- Modify: `crates/agent-ui-bridge/`
- Modify: `docs/cli/`

- [x] **Step 1: Plan CLI/JSON surfaces**

Define stable text/JSON commands for memory snapshots, retrieval preview, proposal inspect/approve/reject, import/export, delete/tombstone, and rebuild/index evidence.

- [x] **Step 2: Implement CLI/JSON surfaces**

Commands must render shared contracts and keep raw sensitive memory out of default summaries.

Completed evidence:

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-cli --test memory_commands
CARGO_TARGET_DIR=target/m64-dev cargo clippy -p agent-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

### Task 4: M64-03 Runtime Memory Influence and Proposal Loop

**Files:**
- Create: `docs/specs/M64-03-runtime-memory-influence-proposals.spec.md`
- Create: `docs/plans/M64-03-runtime-memory-influence-proposals.plan.md`
- Modify: `crates/agent-context/`
- Modify: `crates/agent-core/`
- Modify: `crates/agent-memory/`

- [x] **Step 1: Plan runtime integration**

Define how scoped memories enter prompt assembly, how influence traces attach to turn evidence, and how model output can create reviewable memory write proposals.

- [x] **Step 2: Implement runtime integration**

Runtime memory must be bounded, policy-aware, recoverable, auditable, and explainable through traces.

Completed evidence:

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-context --test memory_prompt_layers
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-core --test runtime_memory_influence
CARGO_TARGET_DIR=target/m64-dev cargo clippy -p agent-context -p agent-core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

### Task 5: M64-04 TUI/Desktop Memory Surfaces

**Files:**
- Create: `docs/specs/M64-04-tui-desktop-memory-surfaces.spec.md`
- Create: `docs/plans/M64-04-tui-desktop-memory-surfaces.plan.md`
- Modify: `crates/agent-tui/`
- Modify: `apps/desktop/`
- Modify: `docs/desktop/`

- [x] **Step 1: Plan UI-safe memory surfaces**

Define memory panels for records, proposals, selected influence, privacy state, import/export state, and settings.

- [x] **Step 2: Implement TUI/Desktop rendering**

TUI and Desktop must render bridge snapshots only and must not implement privacy, proposal, retrieval, or retention decisions.

Completed evidence:

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-ui-bridge --test memory_snapshot
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-tui --test task_panel
CARGO_TARGET_DIR=target/m64-dev cargo clippy -p agent-ui-bridge -p agent-tui --all-targets -- -D warnings
npm --prefix apps/desktop test -- --run src/__tests__/memory-ui.test.tsx
npm --prefix apps/desktop run build
cargo fmt --all -- --check
git diff --check
```

### Task 6: M64-05 Memory Smoke, Docs, and Closeout

**Files:**
- Create: `docs/specs/M64-05-memory-smoke-closeout.spec.md`
- Create: `docs/plans/M64-05-memory-smoke-closeout.plan.md`
- Modify: `docs/eval/`
- Modify: `docs/cli/`
- Modify: `docs/desktop/`
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [ ] **Step 1: Add deterministic smoke coverage**

Add credential-free evidence for retrieval, influence traces, proposal review, import/export, deletion/tombstones, rebuild, redaction, and disabled-memory behavior.

- [ ] **Step 2: Document user workflows**

Document CLI/Desktop memory usage, proposal decisions, influence traces, privacy controls, import/export, deletion, and redaction expectations.

- [ ] **Step 3: Run final M64 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
git diff --check
```

Expected: M64 is productized, documented, smoke-tested, and ready to hand off to M65.

### Task 7: Commit Discipline

**Files:**
- Modify/create only files for the active slice and required status docs.

- [ ] **Step 1: Commit M64 planning**

Exact-stage only M64 milestone and M64-01 planning docs plus milestone index updates.

Suggested commit message:

```text
docs: add m64 memory personalization plan
```

- [ ] **Step 2: Commit each implementation slice**

For each M64 implementation slice, exact-stage the slice spec, slice plan, code, tests, docs, eval fixtures, and status files touched by that slice only.
