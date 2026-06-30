# M58-04 Pinned Context and Budget Planning Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class required/optional pinned project context, budget-aware prompt-layer planning, core orchestration, CLI/JSON evidence, and bridge snapshots while preserving existing retrieval and semantic behavior.

**Architecture:** `agent-context` owns pinned context contracts, validation, metadata-safe prompt layers, and budget decisions. `agent-core` composes workspace scanning, lexical or hybrid retrieval, and pinned budget planning. CLI/TUI/bridge crates render typed reports only.

**Tech Stack:** Rust 2024 edition, serde contracts, existing M58 project summaries/retrieval/hybrid reports, existing `ContextBudget` planner, deterministic fixtures, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M58-04-pinned-context-budget-planning.spec.md`
- Read: `docs/specs/M58-project-context-intelligence.spec.md`
- Read: `docs/specs/M58-02-relevant-file-retrieval.spec.md`
- Read: `docs/specs/M58-03-semantic-hybrid-search.spec.md`
- Read: `crates/agent-context/src/budget.rs`
- Read: `crates/agent-context/src/retrieval.rs`
- Read: `crates/agent-context/src/semantic.rs`
- Read: `crates/agent-core/src/project_context.rs`
- Read: `crates/agent-cli/tests/project_context_retrieval_commands.rs`
- Read: `crates/agent-cli/tests/project_context_semantic_commands.rs`
- Read: `crates/agent-ui-bridge/tests/project_context_retrieval_snapshot.rs`
- Read: `crates/agent-ui-bridge/tests/project_context_semantic_snapshot.rs`

- [ ] **Step 1: Review baseline behavior**

Run:

```bash
cargo test -p agent-context --test budget_planner
cargo test -p agent-context --test context_retrieval
cargo test -p agent-context --test context_semantic_retrieval
cargo test -p agent-core --test project_context_retrieval
cargo test -p agent-core --test project_context_semantic_retrieval
cargo test -p agent-cli --test project_context_retrieval_commands
cargo test -p agent-cli --test project_context_semantic_commands
cargo test -p agent-ui-bridge --test project_context_retrieval_snapshot
cargo test -p agent-ui-bridge --test project_context_semantic_snapshot
```

Expected: existing budget, lexical retrieval, semantic retrieval, CLI, and bridge behavior pass before M58-04 changes.

### Task 2: Write Failing Context Pin Budget Tests

**Files:**
- Create: `crates/agent-context/tests/pinned_context_budget.rs`

- [ ] **Step 1: Add pinned context contract tests**

Cover:

- stable serde shape for pin request, priority, item, omission, diagnostics, and budget report
- path normalization, duplicate rejection, empty path/reason rejection, and invalid budget rejection
- required pins are included before optional pins and retrieval layers
- unknown required pins fail; unknown optional pins are omitted with diagnostics
- required budget overflow fails; optional budget overflow is omitted
- secret-like and metadata-only project files stay metadata-only in reports and prompt layers
- lexical and hybrid retrieval reports can be used as optional retrieval context after pins
- prompt-layer conversion includes pinned metadata and budget status but no raw file contents, secret values, vectors, or raw semantic document text

- [ ] **Step 2: Run context pin tests and verify failure**

Run:

```bash
cargo test -p agent-context --test pinned_context_budget
```

Expected: tests fail before pinned context contracts exist.

### Task 3: Implement `agent-context` Pinned Budget Planning

**Files:**
- Create: `crates/agent-context/src/pins.rs`
- Modify: `crates/agent-context/src/lib.rs`
- Modify if needed: `crates/agent-context/src/budget.rs`
- Modify if needed: `crates/agent-context/src/retrieval.rs`
- Modify if needed: `crates/agent-context/src/semantic.rs`

- [ ] **Step 1: Add typed pinned context contracts**

Implement `PinnedProjectContextRequest`, `PinnedProjectContextPin`, `PinnedContextPriority`, `PinnedContextSource`, `PinnedContextItem`, `PinnedContextOmissionReason`, `PinnedContextOmission`, `PinnedProjectContextBudgetReport`, and safe diagnostics with stable serde names and validation constructors.

- [ ] **Step 2: Add pin resolution**

Resolve pins against `ProjectContextSummary` by normalized workspace-relative path. Preserve request order after normalization. Required unknown paths must fail; optional unknown paths must be omitted with warning diagnostics.

- [ ] **Step 3: Add metadata-safe pinned prompt layers**

Build prompt-layer content from file metadata only: path, kind, language, visibility, size, line count, token estimate, priority, and reason. Do not read raw file contents. Preserve metadata-only visibility for secret-like, binary, generated, ignored, oversized, or metadata-only files.

- [ ] **Step 4: Add budget planning**

Plan required pins first, optional pins second, and retrieval prompt layers last. Reuse existing `ContextBudget` validation and budget class semantics where possible. Required overflow must return a context error; optional overflow and retrieval overflow should become stable omissions.

- [ ] **Step 5: Add retrieval integration**

Accept either lexical `ContextRetrievalReport` or semantic `ContextHybridRetrievalReport` as optional retrieval context. Existing M58-02 and M58-03 report contracts must remain unchanged and usable independently.

- [ ] **Step 6: Run context pin tests**

Run:

```bash
cargo test -p agent-context --test pinned_context_budget
```

Expected: context pinned budget tests pass.

### Task 4: Add Core Scan-Retrieve-Plan Runtime

**Files:**
- Create: `crates/agent-core/tests/project_context_pinned_budget.rs`
- Modify: `crates/agent-core/src/project_context.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [ ] **Step 1: Add core orchestration tests**

Cover:

- scanning a temp workspace, retrieving context, and returning a pinned budget report
- required pin inclusion before retrieval context
- optional pin inclusion when budget fits and omission when budget does not fit
- unknown required pin returning `project_context.retrieval.failed`
- hybrid retrieval plus pinned budget compatibility
- secret-like pinned files staying metadata-only and redacted in serialized reports

- [ ] **Step 2: Implement core helper**

Add typed input/report values that compose `scan_project_context`, lexical or hybrid retrieval, and pinned context budget planning. Keep pin resolution and budget decisions in `agent-context`.

- [ ] **Step 3: Run core pin tests**

Run:

```bash
cargo test -p agent-core --test project_context_pinned_budget
```

Expected: core pinned budget tests pass.

### Task 5: Add CLI Text and JSON Pinned Budget Surface

**Files:**
- Create: `crates/agent-cli/tests/project_context_pinned_commands.rs`
- Modify: `crates/agent-cli/src/command.rs`

- [ ] **Step 1: Add CLI tests**

Cover:

- existing `--pin` retrieval boost remains backward compatible
- `--required-pin <path>` includes a pinned file before retrieval context
- `--optional-pin <path>` includes a pinned file when it fits and reports omission when it does not fit
- `--context-budget <tokens>` and `--reserved-response-tokens <tokens>` control prompt budget planning
- `--json repo context select ...` emits a parseable pinned budget snapshot
- unknown required pin, invalid pin path/reason, invalid budgets, and empty task exit with code `2`
- task text, pin reasons, diagnostics, fixture paths, and secret-like values are bounded and redacted

- [ ] **Step 2: Implement CLI flags**

Extend `repo context select` without changing default lexical behavior:

```text
--required-pin <path>
--optional-pin <path>
--pin-reason <text>
--context-budget <tokens>
--reserved-response-tokens <tokens>
```

Keep existing `--pin` as a retrieval boost compatibility flag. M58-04 pinned budget planning should activate when a required/optional pin or explicit context budget flag is supplied.

- [ ] **Step 3: Run CLI pin tests**

Run:

```bash
cargo test -p agent-cli --test project_context_pinned_commands
```

Expected: CLI pinned budget tests pass.

### Task 6: Add UI Bridge Pinned Budget Snapshot

**Files:**
- Create: `crates/agent-ui-bridge/tests/project_context_pinned_snapshot.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`
- Modify if needed: `crates/agent-tui/src/trace_panel.rs`
- Modify if needed: `crates/agent-tui/src/transcript.rs`

- [ ] **Step 1: Add bridge tests**

Cover:

- stable JSON shape for pinned budget snapshot
- single-line frame encoding
- included pinned item summaries with priority, source, visibility, token estimate, and reason
- omitted pin summaries with stable omission reason and diagnostic code
- retrieval budget inclusion/omission status
- redaction of task text, pin reasons, diagnostics, secret-like file metadata, raw file contents, vectors, and semantic document text

- [ ] **Step 2: Implement bridge mapping**

Add a pinned budget snapshot frame derived from typed core reports. Keep UI summaries bounded and metadata-only.

- [ ] **Step 3: Run bridge pin tests**

Run:

```bash
cargo test -p agent-ui-bridge --test project_context_pinned_snapshot
```

Expected: bridge pinned budget tests pass.

### Task 7: Update Docs, Verify, and Commit

**Files:**
- Modify: `docs/plans/M58-04-pinned-context-budget-planning.plan.md`
- Modify: `docs/plans/M58-project-context-intelligence.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [ ] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-context -p agent-core -p agent-cli -p agent-ui-bridge -p agent-tui --all-targets -- -D warnings
cargo test -p agent-context --test pinned_context_budget
cargo test -p agent-core --test project_context_pinned_budget
cargo test -p agent-cli --test project_context_pinned_commands
cargo test -p agent-ui-bridge --test project_context_pinned_snapshot
```

Expected: focused checks pass.

- [ ] **Step 2: Commit**

Run exact-path staging for the M58-04 source, tests, and docs, then:

```bash
git commit -m "feat: add m58 pinned context budget planning"
```

Expected: commit contains only M58-04 source, tests, and documentation/status updates.

### Completion Checklist

- [ ] Pinned context request/report contracts are typed and serde-tested.
- [ ] Required pins fail safely when unknown or over budget.
- [ ] Optional pins omit safely with stable diagnostics.
- [ ] Pinned context prompt layers are metadata-only and redacted.
- [ ] Retrieval and hybrid retrieval remain independently usable.
- [ ] Core scan-retrieve-plan orchestration exists.
- [ ] CLI text/JSON output exposes safe pinned budget evidence.
- [ ] UI bridge pinned budget snapshot shape is stable and safe.
- [ ] Focused fmt/clippy/tests pass.
