# M58-01 Repo Map and File Tree Summaries Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add bounded project file tree summaries that can feed prompt context and safe CLI/Desktop evidence.

**Architecture:** `agent-context` owns serializable project context contracts and prompt-layer conversion. `agent-core` owns workspace filesystem scanning and limit enforcement. `agent-cli` and `agent-ui-bridge` render typed summaries without duplicating scan logic.

**Tech Stack:** Rust 2024 edition, serde contracts, std filesystem traversal, existing prompt/budget contracts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M58-01-repo-map-file-tree-summaries.spec.md`
- Read: `docs/specs/M58-project-context-intelligence.spec.md`
- Read: `crates/agent-context/src/prompt.rs`
- Read: `crates/agent-context/src/budget.rs`
- Read: `crates/agent-core/src/context.rs`
- Read: `crates/agent-core/src/repo_session.rs`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Review current context behavior**

Run:

```bash
cargo test -p agent-context
cargo test -p agent-core repo_session_bootstrap
cargo test -p agent-ui-bridge prompt_summary
```

Expected: existing prompt, budget, repo session, and UI prompt summary tests pass before M58-01 changes.

Baseline verification on 2026-06-30:

- `cargo test -p agent-context`
- `cargo test -p agent-core --test repo_session_bootstrap`
- `cargo test -p agent-ui-bridge prompt_summary`

### Task 2: Write Failing Context Contract Tests

**Files:**
- Create: `crates/agent-context/tests/project_context_summary.rs`

- [ ] **Step 1: Add serialization and validation tests**

Cover:

- stable JSON shape for `ProjectContextSummary`
- lexicographic sorting of files/directories
- validation rejects empty workspace display, empty paths, duplicate paths, and invalid limits
- secret-like summaries serialize metadata without content previews
- prompt-layer conversion produces a project prompt layer with bounded content and workspace sensitivity

- [ ] **Step 2: Run context tests and verify failure**

Run:

```bash
cargo test -p agent-context project_context_summary
```

Expected: tests fail before project context contracts exist.

### Task 3: Implement `agent-context` Project Contracts

**Files:**
- Create: `crates/agent-context/src/project.rs`
- Modify: `crates/agent-context/src/error.rs`
- Modify: `crates/agent-context/src/lib.rs`

- [ ] **Step 1: Add typed contracts**

Implement project limits, file summaries, directory summaries, omissions, diagnostics, visibility, file kind, validation constructors, and stable serde shapes.

- [ ] **Step 2: Add prompt-layer conversion**

Convert a safe project summary into `PromptLayerKind::Project` content with bounded path lists and no raw secret-like content.

- [ ] **Step 3: Run context tests**

Run:

```bash
cargo test -p agent-context project_context_summary
```

Expected: context tests pass.

### Task 4: Write Failing Core Scan Tests

**Files:**
- Create: `crates/agent-core/tests/project_context_scan.rs`

- [ ] **Step 1: Add bounded workspace scan tests**

Cover:

- nested source/test/config/docs files are summarized with workspace-relative paths
- `.git`, `target`, `node_modules`, and generated directories are omitted
- binary and oversized files become metadata-only omissions
- `.env`, private key, and token-like content do not appear in public summaries
- max files, max depth, and max total bytes produce deterministic diagnostics
- invalid workspace roots fail before scan results are emitted

- [ ] **Step 2: Run core tests and verify failure**

Run:

```bash
cargo test -p agent-core project_context_scan
```

Expected: tests fail before core scanning exists.

### Task 5: Implement Core Workspace Scanner

**Files:**
- Create: `crates/agent-core/src/project_context.rs`
- Modify: `crates/agent-core/src/lib.rs`
- Modify if useful: `crates/agent-core/src/context.rs`

- [ ] **Step 1: Add scan input and report**

Implement a typed scan input with workspace root, limits, optional include/exclude path patterns if needed, and a report wrapping `ProjectContextSummary`.

- [ ] **Step 2: Add bounded traversal**

Use std filesystem APIs, canonical workspace containment checks, deterministic sorting, implicit exclusions, binary/oversized detection, and bounded text inspection for metadata only.

- [ ] **Step 3: Run core tests**

Run:

```bash
cargo test -p agent-core project_context_scan
```

Expected: core scan tests pass.

### Task 6: Add CLI Text and JSON Surface

**Files:**
- Create: `crates/agent-cli/tests/project_context_commands.rs`
- Modify: `crates/agent-cli/src/command.rs`

- [ ] **Step 1: Add CLI tests**

Cover:

- `repo context --workspace .` text output includes file counts and diagnostics
- `--json repo context --workspace .` emits parseable redacted project context frames
- invalid workspace exits with code `2`
- secret-like fixture content is not rendered

- [ ] **Step 2: Implement CLI command**

Add a product-oriented project context command that calls `agent-core` and renders typed output using existing text/JSON patterns.

- [ ] **Step 3: Run CLI tests**

Run:

```bash
cargo test -p agent-cli project_context_commands
```

Expected: CLI tests pass.

### Task 7: Add UI Bridge Snapshot

**Files:**
- Create: `crates/agent-ui-bridge/tests/project_context_snapshot.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify if needed: `crates/agent-ui-bridge/src/lib.rs`

- [ ] **Step 1: Add bridge tests**

Cover stable JSON shape, single-line frame encoding, bounded previews, hidden secret-like details, and deterministic sorting.

- [ ] **Step 2: Implement bridge mapping**

Add a `UiProjectContextSnapshot` or equivalent frame that is derived from `ProjectContextSummary`.

- [ ] **Step 3: Run bridge tests**

Run:

```bash
cargo test -p agent-ui-bridge project_context_snapshot
```

Expected: bridge tests pass.

### Task 8: Verify and Commit

**Files:**
- Modify: `docs/plans/M58-01-repo-map-file-tree-summaries.plan.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-context -p agent-core -p agent-cli -p agent-ui-bridge --all-targets -- -D warnings
cargo test -p agent-context project_context_summary
cargo test -p agent-core project_context_scan
cargo test -p agent-cli project_context_commands
cargo test -p agent-ui-bridge project_context_snapshot
```

Expected: focused checks pass.

- [ ] **Step 2: Commit**

Run exact-path staging for the M58-01 source, tests, and docs, then:

```bash
git commit -m "feat: add m58 repo map summaries"
```

Expected: commit contains only M58-01 source, tests, and documentation/status updates.

### Completion Checklist

- [ ] Project context summary contracts are typed and serde-tested.
- [ ] Project summaries convert into safe project prompt layers.
- [ ] Core workspace scanning is bounded, deterministic, and redacted.
- [ ] CLI text/JSON output exposes safe project context evidence.
- [ ] UI bridge snapshot shape is stable and safe.
- [ ] Focused fmt/clippy/tests pass.
