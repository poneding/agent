# M58-01 Repo Map and File Tree Summaries Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add bounded project file tree summaries that can feed prompt context and safe CLI/Desktop evidence.

**Architecture:** `ruse-context` owns serializable project context contracts and prompt-layer conversion. `ruse-core` owns workspace filesystem scanning and limit enforcement. `ruse-cli` and `ruse-ui-bridge` render typed summaries without duplicating scan logic.

**Tech Stack:** Rust 2024 edition, serde contracts, std filesystem traversal, existing prompt/budget contracts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M58-01-repo-map-file-tree-summaries.spec.md`
- Read: `docs/specs/M58-project-context-intelligence.spec.md`
- Read: `crates/ruse-context/src/prompt.rs`
- Read: `crates/ruse-context/src/budget.rs`
- Read: `crates/ruse-core/src/context.rs`
- Read: `crates/ruse-core/src/repo_session.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`

- [x] **Step 1: Review current context behavior**

Run:

```bash
cargo test -p ruse-context
cargo test -p ruse-core repo_session_bootstrap
cargo test -p ruse-ui-bridge prompt_summary
```

Expected: existing prompt, budget, repo session, and UI prompt summary tests pass before M58-01 changes.

Baseline verification on 2026-06-30:

- `cargo test -p ruse-context`
- `cargo test -p ruse-core --test repo_session_bootstrap`
- `cargo test -p ruse-ui-bridge prompt_summary`

### Task 2: Write Failing Context Contract Tests

**Files:**
- Create: `crates/ruse-context/tests/project_context_summary.rs`

- [x] **Step 1: Add serialization and validation tests**

Cover:

- stable JSON shape for `ProjectContextSummary`
- lexicographic sorting of files/directories
- validation rejects empty workspace display, empty paths, duplicate paths, and invalid limits
- secret-like summaries serialize metadata without content previews
- prompt-layer conversion produces a project prompt layer with bounded content and workspace sensitivity

- [x] **Step 2: Run context tests and verify failure**

Run:

```bash
cargo test -p ruse-context project_context_summary
```

Expected: tests fail before project context contracts exist.

### Task 3: Implement `ruse-context` Project Contracts

**Files:**
- Create: `crates/ruse-context/src/project.rs`
- Modify: `crates/ruse-context/src/error.rs`
- Modify: `crates/ruse-context/src/lib.rs`

- [x] **Step 1: Add typed contracts**

Implement project limits, file summaries, directory summaries, omissions, diagnostics, visibility, file kind, validation constructors, and stable serde shapes.

- [x] **Step 2: Add prompt-layer conversion**

Convert a safe project summary into `PromptLayerKind::Project` content with bounded path lists and no raw secret-like content.

- [x] **Step 3: Run context tests**

Run:

```bash
cargo test -p ruse-context project_context_summary
```

Expected: context tests pass.

### Task 4: Write Failing Core Scan Tests

**Files:**
- Create: `crates/ruse-core/tests/project_context_scan.rs`

- [x] **Step 1: Add bounded workspace scan tests**

Cover:

- nested source/test/config/docs files are summarized with workspace-relative paths
- `.git`, `target`, `node_modules`, and generated directories are omitted
- binary and oversized files become metadata-only omissions
- `.env`, private key, and token-like content do not appear in public summaries
- max files, max depth, and max total bytes produce deterministic diagnostics
- invalid workspace roots fail before scan results are emitted

- [x] **Step 2: Run core tests and verify failure**

Run:

```bash
cargo test -p ruse-core project_context_scan
```

Expected: tests fail before core scanning exists.

### Task 5: Implement Core Workspace Scanner

**Files:**
- Create: `crates/ruse-core/src/project_context.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Modify if useful: `crates/ruse-core/src/context.rs`

- [x] **Step 1: Add scan input and report**

Implement a typed scan input with workspace root, limits, optional include/exclude path patterns if needed, and a report wrapping `ProjectContextSummary`.

- [x] **Step 2: Add bounded traversal**

Use std filesystem APIs, canonical workspace containment checks, deterministic sorting, implicit exclusions, binary/oversized detection, and bounded text inspection for metadata only.

- [x] **Step 3: Run core tests**

Run:

```bash
cargo test -p ruse-core project_context_scan
```

Expected: core scan tests pass.

### Task 6: Add CLI Text and JSON Surface

**Files:**
- Create: `crates/ruse-cli/tests/project_context_commands.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- `repo context --workspace .` text output includes file counts and diagnostics
- `--json repo context --workspace .` emits parseable redacted project context frames
- invalid workspace exits with code `2`
- secret-like fixture content is not rendered

- [x] **Step 2: Implement CLI command**

Add a product-oriented project context command that calls `ruse-core` and renders typed output using existing text/JSON patterns.

- [x] **Step 3: Run CLI tests**

Run:

```bash
cargo test -p ruse-cli project_context_commands
```

Expected: CLI tests pass.

### Task 7: Add UI Bridge Snapshot

**Files:**
- Create: `crates/ruse-ui-bridge/tests/project_context_snapshot.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify if needed: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add bridge tests**

Cover stable JSON shape, single-line frame encoding, bounded previews, hidden secret-like details, and deterministic sorting.

- [x] **Step 2: Implement bridge mapping**

Add a `UiProjectContextSnapshot` or equivalent frame that is derived from `ProjectContextSummary`.

- [x] **Step 3: Run bridge tests**

Run:

```bash
cargo test -p ruse-ui-bridge project_context_snapshot
```

Expected: bridge tests pass.

### Task 8: Verify and Commit

**Files:**
- Modify: `docs/plans/M58-01-repo-map-file-tree-summaries.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-context -p ruse-core -p ruse-cli -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings
cargo test -p ruse-context project_context_summary
cargo test -p ruse-core project_context_scan
cargo test -p ruse-cli project_context_commands
cargo test -p ruse-ui-bridge project_context_snapshot
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging for the M58-01 source, tests, and docs, then:

```bash
git commit -m "feat: add m58 repo map summaries"
```

Expected: commit contains only M58-01 source, tests, and documentation/status updates.

Focused verification on 2026-06-30:

- `cargo fmt --all -- --check`
- `cargo clippy -p ruse-context -p ruse-core -p ruse-cli -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings`
- `cargo test -p ruse-context --test project_context_summary`
- `cargo test -p ruse-core --test project_context_scan`
- `cargo test -p ruse-cli --test project_context_commands`
- `cargo test -p ruse-ui-bridge --test project_context_snapshot`
- `cargo test -p ruse-tui trace_panel`
- `cargo test -p ruse-tui transcript`

Additional workspace verification on 2026-06-30:

- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

### Completion Checklist

- [x] Project context summary contracts are typed and serde-tested.
- [x] Project summaries convert into safe project prompt layers.
- [x] Core workspace scanning is bounded, deterministic, and redacted.
- [x] CLI text/JSON output exposes safe project context evidence.
- [x] UI bridge snapshot shape is stable and safe.
- [x] Focused fmt/clippy/tests pass.
