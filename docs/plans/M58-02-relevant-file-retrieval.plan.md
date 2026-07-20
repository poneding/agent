# M58-02 Relevant File Retrieval Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic lexical relevant-file retrieval over M58-01 project summaries, with selected/omitted candidates, scoring reasons, budget behavior, and safe CLI/bridge evidence.

**Architecture:** `ruse-context` owns retrieval contracts, scoring, selection, and prompt-layer conversion. `ruse-core` composes workspace scanning with retrieval. CLI/JSON and bridge surfaces render typed reports without duplicating retrieval logic.

**Tech Stack:** Rust 2024 edition, serde contracts, M58-01 project summaries, existing prompt/budget contracts, deterministic temp fixtures, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M58-02-relevant-file-retrieval.spec.md`
- Read: `docs/specs/M58-project-context-intelligence.spec.md`
- Read: `crates/ruse-context/src/project.rs`
- Read: `crates/ruse-core/src/project_context.rs`
- Read: `crates/ruse-cli/tests/project_context_commands.rs`
- Read: `crates/ruse-ui-bridge/tests/project_context_snapshot.rs`

- [x] **Step 1: Review M58-01 baseline**

Run:

```bash
cargo test -p ruse-context --test project_context_summary
cargo test -p ruse-core --test project_context_scan
cargo test -p ruse-cli --test project_context_commands
cargo test -p ruse-ui-bridge --test project_context_snapshot
```

Expected: M58-01 project summaries and context command surfaces pass before retrieval changes.

### Task 2: Write Failing Context Retrieval Tests

**Files:**
- Create: `crates/ruse-context/tests/context_retrieval.rs`

- [x] **Step 1: Add retrieval contract tests**

Cover:

- stable serde shape for query, selected candidates, omissions, diagnostics, and reasons
- lexical scoring from task words against path, filename, directory, language, and kind
- deterministic ordering by score descending then path ascending
- pinned paths boost candidates and unknown pins produce diagnostics
- budget-limited candidates are omitted with `budget_exceeded`
- selected candidates convert into safe project prompt layers

- [x] **Step 2: Run context tests and verify failure**

Run:

```bash
cargo test -p ruse-context --test context_retrieval
```

Expected: tests fail before retrieval contracts exist.

### Task 3: Implement `ruse-context` Retrieval

**Files:**
- Create: `crates/ruse-context/src/retrieval.rs`
- Modify: `crates/ruse-context/src/lib.rs`
- Modify if needed: `crates/ruse-context/src/error.rs`

- [x] **Step 1: Add typed retrieval contracts**

Implement query, report, candidate, omission, diagnostic, and reason types with validation constructors and stable serde names.

- [x] **Step 2: Add lexical scoring**

Tokenize task text and compare against project summary path components, filenames, directory names, language, and file kind. Keep scoring deterministic and bounded.

- [x] **Step 3: Add selection and prompt-layer conversion**

Select candidates under budget, omit over-budget candidates with reasons, and produce summary-only project prompt layers from selected metadata.

- [x] **Step 4: Run context retrieval tests**

Run:

```bash
cargo test -p ruse-context --test context_retrieval
```

Expected: context retrieval tests pass.

### Task 4: Add Core Scan-Plus-Retrieve Runtime

**Files:**
- Create: `crates/ruse-core/tests/project_context_retrieval.rs`
- Modify: `crates/ruse-core/src/project_context.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add core orchestration tests**

Cover scanning a temp workspace, retrieving relevant files for a task, preserving safe paths, returning unknown-pin diagnostics, and keeping secret-like files metadata-only.

- [x] **Step 2: Implement core helper**

Add a typed input/report that composes `scan_project_context` with `ruse-context` retrieval.

- [x] **Step 3: Run core tests**

Run:

```bash
cargo test -p ruse-core --test project_context_retrieval
```

Expected: core retrieval tests pass.

### Task 5: Add CLI Text and JSON Retrieval Surface

**Files:**
- Create: `crates/ruse-cli/tests/project_context_retrieval_commands.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- `repo context select --workspace . --task "..."` text output lists selected files and reasons
- `--json repo context select --workspace . --task "..."` emits a parseable retrieval snapshot
- `--pin path` boosts a matching path
- invalid workspace and empty task exit with code `2`
- task text and diagnostics are bounded and redacted

- [x] **Step 2: Implement CLI command**

Extend the `repo context` namespace with a retrieval subcommand that calls shared core retrieval.

- [x] **Step 3: Run CLI tests**

Run:

```bash
cargo test -p ruse-cli --test project_context_retrieval_commands
```

Expected: CLI retrieval tests pass.

### Task 6: Add UI Bridge Retrieval Snapshot

**Files:**
- Create: `crates/ruse-ui-bridge/tests/project_context_retrieval_snapshot.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Modify if needed: `crates/ruse-tui/src/trace_panel.rs`
- Modify if needed: `crates/ruse-tui/src/transcript.rs`

- [x] **Step 1: Add bridge tests**

Cover stable JSON shape, single-line frame encoding, selected/omitted candidate summaries, diagnostics, and redaction.

- [x] **Step 2: Implement bridge mapping**

Add a retrieval snapshot frame derived from the typed retrieval report.

- [x] **Step 3: Run bridge tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test project_context_retrieval_snapshot
```

Expected: bridge retrieval tests pass.

### Task 7: Verify and Commit

**Files:**
- Modify: `docs/plans/M58-02-relevant-file-retrieval.plan.md`
- Modify: `docs/plans/M58-project-context-intelligence.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-context -p ruse-core -p ruse-cli -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings
cargo test -p ruse-context --test context_retrieval
cargo test -p ruse-core --test project_context_retrieval
cargo test -p ruse-cli --test project_context_retrieval_commands
cargo test -p ruse-ui-bridge --test project_context_retrieval_snapshot
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging for the M58-02 source, tests, and docs, then:

```bash
git commit -m "feat: add m58 relevant file retrieval"
```

Expected: commit contains only M58-02 source, tests, and documentation/status updates.

### Completion Checklist

- [x] Retrieval query/report contracts are typed and serde-tested.
- [x] Lexical scoring is deterministic and reasoned.
- [x] Budgeted selected/omitted candidate behavior is tested.
- [x] Core scan-plus-retrieve orchestration exists.
- [x] CLI text/JSON output exposes safe retrieval evidence.
- [x] UI bridge retrieval snapshot shape is stable and safe.
- [x] Focused fmt/clippy/tests pass.
