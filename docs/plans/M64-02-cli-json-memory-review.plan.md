# M64-02 CLI and JSON Memory Review Commands Plan

> **For agentic workers:** Keep memory privacy and product logic in `ruse-memory`. `ruse-cli` should parse commands, call shared memory contracts, and render text/JSON only.

**Goal:** Add stable text and JSON CLI surfaces for memory snapshots, retrieval previews, proposal review/decisions, import/export, deletion/tombstones, and index rebuild evidence.

**Architecture:** `ruse-cli` owns parsing and rendering. `MemoryCommandRuntime` adapts deterministic fixture-backed memory records/proposals/retrieval into `ruse-memory` product snapshots. Later slices will wire runtime prompt influence and UI bridge/Desktop rendering.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M64-memory-personalization.spec.md`
- Read: `docs/specs/M64-01-memory-product-snapshot-privacy.spec.md`
- Read: `docs/specs/M64-02-cli-json-memory-review.spec.md`
- Read: `crates/ruse-memory/src/product.rs`
- Read: `crates/ruse-memory/src/proposal.rs`
- Read: `crates/ruse-memory/src/rerank.rs`
- Read: `crates/ruse-memory/src/editor.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-cli/tests/memory_commands.rs`

- [x] **Step 1: Review current memory CLI baseline**

Confirm existing fixture-backed CLI commands cover list, inspect, delete, export, import, and rebuild-index, but do not yet expose product snapshots, retrieval previews, or proposal decisions.

### Task 2: Write Focused CLI Tests

**Files:**
- Modify: `crates/ruse-cli/tests/memory_commands.rs`

- [x] **Step 1: Add parse tests**

Cover:

- `memory snapshot`
- `memory retrieve --query <text> [--max-selected <n>]`
- `memory proposal list`
- `memory proposal inspect <id>`
- `memory proposal approve <id>`
- `memory proposal reject <id> --reason <reason>`
- `memory proposal expire <id> --reason <reason>`
- `memory proposal commit <id>`

- [x] **Step 2: Add render tests**

Cover text and JSON output for:

- product snapshot
- retrieval preview and influence redaction
- proposal list/inspect
- proposal approve/reject/expire/commit
- invalid proposal transitions
- redacted errors

- [x] **Step 3: Run focused test and verify failure before implementation**

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-cli --test memory_commands
```

Expected: new tests fail until parser/runtime/rendering support is implemented.

### Task 3: Implement Parser and Runtime Support

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Extend `MemoryCommand`**

Add command variants for snapshot, retrieval preview, and proposal review/decision actions.

- [x] **Step 2: Extend parsing**

Parse the new subcommands with required values and stable error messages. Reject invalid `--max-selected` and missing `--reason` values.

- [x] **Step 3: Extend `MemoryCommandRuntime`**

Add deterministic proposal fixture state and retrieval preview helpers. Runtime adapters should rebuild retrieval candidates from active records and call `ruse-memory` rerank/product contracts rather than implementing privacy rules locally.

### Task 4: Implement Text and JSON Rendering

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Render text review workflows**

Render compact text summaries for snapshots, retrieval previews, proposal lists, proposal decisions, and commits.

- [x] **Step 2: Render stable JSON frames**

Emit stable JSON frame `type` values documented in the spec. Include shared product contracts where possible.

- [x] **Step 3: Preserve existing command behavior**

Existing `list`, `inspect`, `delete`, `export`, `import`, and `rebuild-index` tests must keep passing.

### Task 5: Docs and Status

**Files:**
- Modify: `docs/cli/`
- Modify: `docs/plans/M64-memory-personalization.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Add CLI memory docs**

Document the memory review workflow, JSON frame types, proposal decisions, retrieval previews, import/export, deletion/tombstones, rebuild evidence, and privacy guarantees.

- [x] **Step 2: Update status docs after focused gates pass**

Mark M64-02 complete only after focused checks pass. Keep M64 overall in progress.

### Task 6: Verify and Commit

**Files:**
- Modify/create only M64-02 files and required status docs.

- [x] **Step 1: Run focused checks**

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p ruse-cli --test memory_commands
CARGO_TARGET_DIR=target/m64-dev cargo clippy -p ruse-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 2: Commit**

Exact-stage only M64-02 files and required status docs.

Suggested commit message:

```text
feat: add m64 memory review cli commands
```
