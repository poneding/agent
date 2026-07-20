# M58-03 Semantic and Hybrid Search Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add optional embedding-backed project context retrieval with deterministic hybrid scoring, safe lexical fallback, CLI/JSON evidence, and bridge snapshots.

**Architecture:** `ruse-context` owns semantic/hybrid contracts, safe semantic document construction, embedding request mapping, score fusion, and prompt-layer conversion. `ruse-core` composes workspace scanning with optional semantic retrieval. CLI/JSON and bridge surfaces render typed reports without duplicating retrieval logic.

**Tech Stack:** Rust 2024 edition, serde contracts, M58-01 project summaries, M58-02 lexical retrieval, `ruse-memory` embedding provider contracts, deterministic fixture embeddings, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M58-03-semantic-hybrid-search.spec.md`
- Read: `docs/specs/M58-02-relevant-file-retrieval.spec.md`
- Read: `docs/specs/M58-project-context-intelligence.spec.md`
- Read: `crates/ruse-context/src/retrieval.rs`
- Read: `crates/ruse-core/src/project_context.rs`
- Read: `crates/ruse-memory/src/embedding.rs`
- Read: `crates/ruse-memory/src/index.rs`
- Read: `crates/ruse-cli/tests/project_context_retrieval_commands.rs`
- Read: `crates/ruse-ui-bridge/tests/project_context_retrieval_snapshot.rs`

- [x] **Step 1: Review semantic retrieval inputs**

Run:

```bash
cargo test -p ruse-context --test context_retrieval
cargo test -p ruse-core --test project_context_retrieval
cargo test -p ruse-memory --test embedding_provider
cargo test -p ruse-memory --test memory_index_hybrid_query
cargo test -p ruse-cli --test project_context_retrieval_commands
cargo test -p ruse-ui-bridge --test project_context_retrieval_snapshot
```

Expected: lexical retrieval and existing embedding/hybrid memory fixtures pass before M58-03 changes.

### Task 2: Write Failing Context Semantic Retrieval Tests

**Files:**
- Create: `crates/ruse-context/tests/context_semantic_retrieval.rs`

- [x] **Step 1: Add semantic/hybrid contract tests**

Cover:

- stable serde shape for semantic config, status, hybrid candidates, omissions, and diagnostics
- validation rejects invalid provider ids, dimensions, weights, minimum score, and fallback policy misuse
- safe semantic document construction skips or metadata-limits secret-like and metadata-only files
- deterministic fixture embeddings can change ranking when semantic weight dominates
- disabled or failed semantic retrieval returns lexical-only results with safe diagnostics
- hybrid candidates sort by combined score descending, semantic score descending, lexical score descending, then path ascending
- prompt-layer conversion includes scores/status but not raw vectors, semantic document text, or secret-like task text

- [x] **Step 2: Run context semantic tests and verify failure**

Run:

```bash
cargo test -p ruse-context --test context_semantic_retrieval
```

Expected: tests fail before semantic retrieval contracts exist.

### Task 3: Implement `ruse-context` Semantic and Hybrid Retrieval

**Files:**
- Create: `crates/ruse-context/src/semantic.rs`
- Modify: `crates/ruse-context/src/retrieval.rs`
- Modify: `crates/ruse-context/src/lib.rs`
- Modify if needed: `crates/ruse-context/src/error.rs`
- Modify if needed: `crates/ruse-context/Cargo.toml`

- [x] **Step 1: Add typed semantic contracts**

Implement semantic config, mode, fallback policy, semantic summary, semantic document, hybrid candidate, hybrid omission, and hybrid report contracts with stable serde names and validation constructors.

- [x] **Step 2: Add safe semantic document construction**

Build bounded semantic document text from workspace-relative path, filename, directories, kind, language, visibility, and safe metadata. Do not include raw secret-like content, raw file contents, or embedding vectors in public contracts.

- [x] **Step 3: Add embedding request mapping**

Use `ruse-memory::EmbeddingProvider`, `EmbeddingRequest`, and `EmbeddingInput` to embed the redacted task query plus safe project semantic documents. Preserve deterministic input ids and reject duplicate ids before provider execution.

- [x] **Step 4: Add hybrid score fusion**

Normalize M58-02 lexical scores, combine optional semantic cosine scores with configured weights, preserve pinned-path influence, and sort candidates deterministically. Reuse M58-02 budget selection and omission semantics.

- [x] **Step 5: Add lexical fallback**

When semantic mode is disabled or provider execution fails with `LexicalOnly`, return a hybrid report based on M58-02 lexical retrieval plus safe diagnostics. When configured as `FailClosed`, return a retrieval error.

- [x] **Step 6: Run context semantic tests**

Run:

```bash
cargo test -p ruse-context --test context_semantic_retrieval
```

Expected: context semantic retrieval tests pass.

### Task 4: Add Core Scan-Plus-Hybrid Runtime

**Files:**
- Create: `crates/ruse-core/tests/project_context_semantic_retrieval.rs`
- Modify: `crates/ruse-core/src/project_context.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add core semantic orchestration tests**

Cover:

- scanning a temp workspace and returning hybrid retrieval reports
- deterministic fixture semantic retrieval changing candidate order when configured
- disabled semantic mode preserving lexical results
- provider failure producing lexical fallback diagnostics
- fail-closed provider failure returning status code `project_context.retrieval.failed`
- secret-like files staying metadata-only and not being embedded as raw content

- [x] **Step 2: Implement core helper**

Add typed input/report values that compose `scan_project_context` with `ruse-context` hybrid retrieval and optional embedding provider/config. Keep provider execution and score fusion out of CLI/UI crates.

- [x] **Step 3: Run core semantic tests**

Run:

```bash
cargo test -p ruse-core --test project_context_semantic_retrieval
```

Expected: core semantic retrieval tests pass.

### Task 5: Add CLI Text and JSON Hybrid Retrieval Surface

**Files:**
- Create: `crates/ruse-cli/tests/project_context_semantic_commands.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- `repo context select --workspace . --task "..." --semantic off` preserves lexical output
- `repo context select --workspace . --task "..." --semantic fixture` lists hybrid scores and semantic status in text output
- `--json repo context select --workspace . --task "..." --semantic fixture` emits a parseable hybrid retrieval snapshot
- invalid semantic dimensions, weights, or empty task exit with code `2`
- task text, diagnostics, and fixture paths are bounded and redacted

- [x] **Step 2: Implement CLI command flags**

Extend `repo context select` with semantic flags without changing the lexical default:

```text
--semantic off|fixture
--semantic-dimensions <n>
--lexical-weight <float>
--semantic-weight <float>
--minimum-score <float>
--semantic-fallback lexical|fail-closed
```

Use deterministic fixture embeddings for CLI tests. Leave real provider account/config onboarding out of this slice.

- [x] **Step 3: Run CLI semantic tests**

Run:

```bash
cargo test -p ruse-cli --test project_context_semantic_commands
```

Expected: CLI semantic retrieval tests pass.

### Task 6: Add UI Bridge Hybrid Retrieval Snapshot

**Files:**
- Create: `crates/ruse-ui-bridge/tests/project_context_semantic_snapshot.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Modify if needed: `crates/ruse-tui/src/trace_panel.rs`
- Modify if needed: `crates/ruse-tui/src/transcript.rs`

- [x] **Step 1: Add bridge tests**

Cover:

- stable JSON shape for hybrid retrieval snapshot
- single-line frame encoding
- selected/omitted hybrid candidate summaries with lexical, semantic, and combined scores
- semantic status and diagnostics
- redaction of task text and absence of raw vectors or raw semantic document text

- [x] **Step 2: Implement bridge mapping**

Add a hybrid retrieval snapshot frame derived from typed core reports. Keep UI summaries bounded and metadata-only.

- [x] **Step 3: Run bridge semantic tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test project_context_semantic_snapshot
```

Expected: bridge semantic retrieval tests pass.

### Task 7: Update Docs, Verify, and Commit

**Files:**
- Modify: `docs/plans/M58-03-semantic-hybrid-search.plan.md`
- Modify: `docs/plans/M58-project-context-intelligence.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-context -p ruse-core -p ruse-cli -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings
cargo test -p ruse-context --test context_semantic_retrieval
cargo test -p ruse-core --test project_context_semantic_retrieval
cargo test -p ruse-cli --test project_context_semantic_commands
cargo test -p ruse-ui-bridge --test project_context_semantic_snapshot
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging for the M58-03 source, tests, and docs, then:

```bash
git commit -m "feat: add m58 semantic hybrid retrieval"
```

Expected: commit contains only M58-03 source, tests, and documentation/status updates.

### Completion Checklist

- [x] Semantic config/report contracts are typed and serde-tested.
- [x] Safe semantic document construction avoids raw secret-like content.
- [x] Deterministic fixture embeddings influence hybrid ranking when configured.
- [x] Lexical retrieval remains the default and fallback path.
- [x] Core scan-plus-hybrid orchestration exists.
- [x] CLI text/JSON output exposes safe hybrid retrieval evidence.
- [x] UI bridge hybrid snapshot shape is stable and safe.
- [x] Focused fmt/clippy/tests pass.
