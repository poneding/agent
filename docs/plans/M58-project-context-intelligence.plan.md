# M58 Project Context Intelligence Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add repo-aware context assembly for coding tasks, including project summaries, relevant-file retrieval, pinned context, budget decisions, and context quality evidence.

**Architecture:** `agent-context` owns typed context contracts, retrieval reports, prompt-layer conversion, and budget decisions. `agent-core` owns workspace scanning and runtime orchestration. CLI/JSON and bridge surfaces consume typed summaries without duplicating context logic.

**Tech Stack:** Rust 2024 edition, serde contracts, deterministic fixtures, existing prompt/budget runtime, Cargo fmt/clippy/test, `cargo xtask eval`.

---

### Task 1: Confirm M58 Inputs

**Files:**
- Read: `docs/specs/M58-project-context-intelligence.spec.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/testing-strategy.md`
- Read: `crates/agent-context/src/lib.rs`
- Read: `crates/agent-context/src/prompt.rs`
- Read: `crates/agent-context/src/budget.rs`
- Read: `crates/agent-core/src/context.rs`
- Read: `crates/agent-core/src/repo_session.rs`

- [x] **Step 1: Review context baseline**

Run:

```bash
cargo test -p agent-context
cargo test -p agent-core m05_context_memory_integration
cargo test -p agent-ui-bridge prompt_summary
```

Expected: current prompt layers, budget planning, memory prompt conversion, and bridge prompt summaries pass before M58 changes.

Baseline verification on 2026-06-30:

- `cargo test -p agent-context`
- `cargo test -p agent-core m05_context_memory_integration`
- `cargo test -p agent-ui-bridge prompt_summary`

### Task 2: M58-01 Repo Map and File Tree Summaries

**Files:**
- Execute: `docs/specs/M58-01-repo-map-file-tree-summaries.spec.md`
- Execute: `docs/plans/M58-01-repo-map-file-tree-summaries.plan.md`

- [x] **Step 1: Complete bounded project summary slice**

Add typed project file/directory summaries, bounded workspace scanning in shared runtime code, prompt-layer conversion, CLI/JSON output, bridge snapshots, and redaction tests.

### Task 3: M58-02 Relevant File Retrieval

**Files:**
- Execute: `docs/specs/M58-02-relevant-file-retrieval.spec.md`
- Execute: `docs/plans/M58-02-relevant-file-retrieval.plan.md`

- [x] **Step 1: Complete lexical retrieval slice**

Rank relevant files from task text, paths, symbols, file metadata, and existing project summaries with deterministic scoring reasons and budget-aware candidate limits.

### Task 4: M58-03 Semantic and Hybrid Search

**Files:**
- Execute: `docs/specs/M58-03-semantic-hybrid-search.spec.md`
- Execute: `docs/plans/M58-03-semantic-hybrid-search.plan.md`

- [ ] **Step 1: Complete optional semantic retrieval slice**

Integrate configured embedding-backed retrieval where available, keep lexical retrieval as the deterministic fallback, and add fixture-backed hybrid scoring tests.

### Task 5: M58-04 Pinned Context and Budget Planning

**Files:**
- Create: `docs/specs/M58-04-pinned-context-budget-planning.spec.md`
- Create: `docs/plans/M58-04-pinned-context-budget-planning.plan.md`

- [ ] **Step 1: Complete pinned context slice**

Add typed pinned context requests, required/optional pin behavior, budget diagnostics, CLI/JSON controls, and prompt-layer integration.

### Task 6: M58-05 Context Quality Traces and Eval

**Files:**
- Create: `docs/specs/M58-05-context-quality-traces-eval.spec.md`
- Create: `docs/plans/M58-05-context-quality-traces-eval.plan.md`

- [ ] **Step 1: Complete quality/eval slice**

Add context quality traces, UI-safe summaries, deterministic eval fixture coverage, and default-suite integration if the scenario is stable enough for the main gate.

### Task 7: Final Verification and Closeout

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M58-project-context-intelligence.plan.md`

- [ ] **Step 1: Run final M58 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask eval
```

Expected: all gates pass with context summaries and eval artifacts redacted.

- [ ] **Step 2: Commit M58 closeout**

Run exact-path staging and commit the final docs/status updates.

### Completion Checklist

- [x] Project file tree summaries are typed, bounded, serde-friendly, and tested.
- [x] Relevant-file retrieval is deterministic and budget-aware.
- [ ] Semantic/hybrid retrieval is optional and fixture-tested.
- [ ] Pinned context participates in prompt budget planning with diagnostics.
- [ ] CLI/JSON and bridge surfaces expose safe context evidence.
- [ ] Context quality traces and eval coverage are implemented.
- [ ] Final workspace fmt/clippy/tests and `cargo xtask eval` pass.
