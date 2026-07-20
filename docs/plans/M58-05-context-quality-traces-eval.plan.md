# M58-05 Context Quality Traces and Eval Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed context quality traces, safe CLI/JSON and bridge evidence, and deterministic eval coverage for M58 project context assembly.

**Architecture:** `ruse-context` owns quality trace contracts and derivation from existing context reports. `ruse-core` composes workspace scan/retrieve/plan helpers. CLI/TUI/bridge render typed summaries only. `ruse-eval` runs deterministic fixture scenarios and writes redacted artifacts.

**Tech Stack:** Rust 2024 edition, serde contracts, existing M58 project/retrieval/semantic/pinned reports, existing eval manifest/runner/golden infrastructure, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask eval`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M58-05-context-quality-traces-eval.spec.md`
- Read: `docs/specs/M58-project-context-intelligence.spec.md`
- Read: `docs/specs/M58-02-relevant-file-retrieval.spec.md`
- Read: `docs/specs/M58-03-semantic-hybrid-search.spec.md`
- Read: `docs/specs/M58-04-pinned-context-budget-planning.spec.md`
- Read: `crates/ruse-context/src/retrieval.rs`
- Read: `crates/ruse-context/src/semantic.rs`
- Read: `crates/ruse-context/src/pins.rs`
- Read: `crates/ruse-core/src/project_context.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-eval/src/manifest.rs`
- Read: `crates/ruse-eval/src/runner.rs`
- Read: `docs/eval/default.suite.json`

- [x] **Step 1: Review current context and eval behavior**

Run:

```bash
cargo test -p ruse-context --test context_retrieval
cargo test -p ruse-context --test context_semantic_retrieval
cargo test -p ruse-context --test pinned_context_budget
cargo test -p ruse-core --test project_context_pinned_budget
cargo test -p ruse-cli --test project_context_pinned_commands
cargo test -p ruse-ui-bridge --test project_context_pinned_snapshot
cargo test -p ruse-eval --test runner
cargo test -p ruse-eval --test golden
cargo xtask eval
```

Expected: existing context retrieval, pinned budget, eval runner, golden comparison, and default eval suite pass before M58-05 changes.

### Task 2: Write Failing Context Quality Contract Tests

**Files:**
- Create: `crates/ruse-context/tests/context_quality_traces.rs`

- [x] **Step 1: Add quality trace tests**

Cover:

- stable serde shape for trace, retrieval kind, budget pressure, signals, diagnostics, and severity
- budget-pressure classification from used/available tokens
- lexical retrieval trace derivation from `ContextRetrievalReport`
- hybrid retrieval trace derivation from `ContextHybridRetrievalReport`
- pinned lexical and pinned hybrid trace derivation from `PinnedProjectContextBudgetReport`
- required pins, optional pins, selected candidates, omitted candidates, retrieval inclusion, retrieval omission, semantic completion, and semantic fallback become deterministic signals
- signal ordering is deterministic by signal kind, request/report order, path, and message
- task text, paths, pin reasons, diagnostics, raw file contents, raw semantic document text, and vectors are redacted or absent

- [x] **Step 2: Run quality trace tests and verify failure**

Run:

```bash
cargo test -p ruse-context --test context_quality_traces
```

Expected: tests fail before quality trace contracts exist.

### Task 3: Implement `ruse-context` Quality Traces

**Files:**
- Create: `crates/ruse-context/src/quality.rs`
- Modify: `crates/ruse-context/src/lib.rs`
- Modify if needed: `crates/ruse-context/src/retrieval.rs`
- Modify if needed: `crates/ruse-context/src/semantic.rs`
- Modify if needed: `crates/ruse-context/src/pins.rs`

- [x] **Step 1: Add typed quality contracts**

Implement `ContextQualityTrace`, `ContextQualityRetrievalKind`, `ContextQualityBudgetPressure`, `ContextQualitySignal`, `ContextQualitySignalKind`, `ContextQualityDiagnosticSeverity`, and `ContextQualityDiagnostic` with stable serde names and validation constructors.

- [x] **Step 2: Add quality derivation helpers**

Build lexical, hybrid, pinned lexical, and pinned hybrid quality traces from existing reports. Do not rescan files, rerun retrieval, or duplicate scoring logic.

- [x] **Step 3: Add redaction and bounded output**

Ensure task text, diagnostic messages, paths, pin reasons, and signal messages are bounded and redacted. Do not include raw file contents, raw semantic documents, vectors, absolute workspace roots, or secret-like values.

- [x] **Step 4: Run context quality tests**

Run:

```bash
cargo test -p ruse-context --test context_quality_traces
```

Expected: context quality trace tests pass.

### Task 4: Add Core Quality Orchestration

**Files:**
- Create: `crates/ruse-core/tests/project_context_quality_traces.rs`
- Modify: `crates/ruse-core/src/project_context.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add core orchestration tests**

Cover:

- scanning a temp workspace, running lexical retrieval, and returning a quality trace
- hybrid retrieval quality trace preserving semantic status evidence
- pinned lexical quality trace preserving required/optional pin budget evidence
- pinned hybrid quality trace preserving both semantic and pinned budget evidence
- invalid workspace or invalid required pins fail before trace construction
- serialized quality reports stay workspace-relative and redacted

- [x] **Step 2: Implement core helpers**

Add typed input/report values or helper functions that compose existing M58 retrieval helpers with quality trace construction. Keep all quality decisions in `ruse-context`.

- [x] **Step 3: Run core quality tests**

Run:

```bash
cargo test -p ruse-core --test project_context_quality_traces
```

Expected: core quality trace tests pass.

### Task 5: Add CLI Text and JSON Quality Surface

**Files:**
- Create: `crates/ruse-cli/tests/project_context_quality_commands.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- default `repo context select` output remains unchanged without quality flags
- `repo context select --quality-trace` appends bounded text quality evidence
- `--json repo context select --quality-trace` emits the existing context snapshot plus a quality trace frame and `Done`
- lexical, hybrid, pinned lexical, and pinned hybrid quality traces are reachable through existing flags
- invalid quality requests reuse existing exit code `2` behavior
- task text, diagnostics, fixture paths, pin reasons, and secret-like values are redacted

- [x] **Step 2: Implement CLI flag and routing**

Extend `repo context select` with:

```text
--quality-trace
```

Keep existing lexical, semantic, and pinned behavior unchanged unless the flag is present. Reuse core quality helpers; do not compute quality in CLI renderers.

- [x] **Step 3: Run CLI quality tests**

Run:

```bash
cargo test -p ruse-cli --test project_context_quality_commands
```

Expected: CLI quality trace tests pass.

### Task 6: Add UI Bridge Quality Snapshot

**Files:**
- Create: `crates/ruse-ui-bridge/tests/project_context_quality_snapshot.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Modify: `crates/ruse-tui/src/trace_panel.rs`
- Modify: `crates/ruse-tui/src/transcript.rs`

- [x] **Step 1: Add bridge tests**

Cover:

- stable JSON shape for quality trace snapshot
- single-line frame encoding
- bounded signal and diagnostic summaries
- budget pressure, retrieval kind, selected/pinned counts, omitted counts, and token counts
- redaction of task text, signal messages, diagnostics, fixture paths, raw contents, vectors, and secret-like values

- [x] **Step 2: Implement bridge mapping**

Add a `UiJsonFrame` variant and UI summary structs derived from typed core/context quality reports. Update TUI trace/transcript reducers for compact display.

- [x] **Step 3: Run bridge quality tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test project_context_quality_snapshot
```

Expected: bridge quality snapshot tests pass.

### Task 7: Add Deterministic Context Quality Eval

**Files:**
- Create: `crates/ruse-eval/tests/context_quality.rs`
- Create: `docs/eval/scenarios/m58-context-quality.eval.json`
- Create: `docs/eval/fixtures/m58-context-quality-workspace/README.md`
- Create as needed: additional files under `docs/eval/fixtures/m58-context-quality-workspace/`
- Create if stable: `docs/eval/goldens/default-m58-context-quality.golden.json`
- Modify: `crates/ruse-eval/src/manifest.rs`
- Modify: `crates/ruse-eval/src/runner.rs`
- Modify: `crates/ruse-eval/src/lib.rs`
- Modify: `docs/eval/default.suite.json`

- [x] **Step 1: Add eval tests and fixture**

Cover:

- manifest validation for context-quality expectations
- pass case when required selected and pinned paths appear in the quality trace
- fail case when expected paths are missing
- redacted diagnostics and artifacts
- optional default-suite/golden integration when output is stable

- [x] **Step 2: Implement eval scenario support**

Add a deterministic context-quality scenario kind that runs core context selection over the fixture workspace and compares the resulting trace against typed expectations.

- [x] **Step 3: Run eval tests and default suite**

Run:

```bash
cargo test -p ruse-eval --test context_quality
cargo xtask eval
```

Expected: context-quality eval tests pass, and the default suite passes if the M58 scenario is included.

### Task 8: Update Docs, Verify, and Commit

**Files:**
- Modify: `docs/plans/M58-05-context-quality-traces-eval.plan.md`
- Modify: `docs/plans/M58-project-context-intelligence.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify if needed: `README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-context -p ruse-core -p ruse-cli -p ruse-ui-bridge -p ruse-tui -p ruse-eval --all-targets -- -D warnings
cargo test -p ruse-context --test context_quality_traces
cargo test -p ruse-core --test project_context_quality_traces
cargo test -p ruse-cli --test project_context_quality_commands
cargo test -p ruse-ui-bridge --test project_context_quality_snapshot
cargo test -p ruse-eval --test context_quality
cargo xtask eval
```

Expected: focused checks and eval suite pass.

- [x] **Step 2: Commit**

Run exact-path staging for the M58-05 source, tests, eval fixtures, and docs, then:

```bash
git commit -m "feat: add m58 context quality traces"
```

Expected: commit contains only M58-05 source, tests, eval fixtures, and documentation/status updates.

### Completion Checklist

- [x] Quality trace contracts are typed and serde-tested.
- [x] Lexical, hybrid, pinned lexical, and pinned hybrid reports produce deterministic traces.
- [x] Quality traces include selected, omitted, pinned, budget, semantic, and diagnostic evidence.
- [x] CLI text/JSON output exposes quality evidence only when requested.
- [x] UI bridge quality snapshot shape is stable and safe.
- [x] Context quality eval scenario is deterministic and credential-free.
- [x] Default eval suite includes M58 quality coverage if stable.
- [x] Focused fmt/clippy/tests and `cargo xtask eval` pass.
