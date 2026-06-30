# M56-02 Edit and Diff Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a typed, policy-routed edit/diff flow that previews or applies a workspace file edit and returns safe changed-file/diff evidence for CLI/JSON and UI bridge consumers.

**Architecture:** `agent-core` owns the flow contract and orchestration. `agent-tool` remains the source of file edit and Git diff/status behavior. `agent-cli` renders command surfaces. `agent-ui-bridge` owns safe diff snapshots. `agent-tui` consumes bridge evidence. Runtime logic must not move into UI crates.

**Tech Stack:** Rust 2024 edition, serde, existing `ToolLoop`, `WorkspaceSandbox`, `StaticPolicyEngine`, `FileEditTool`, `GitStatusTool`, `GitDiffTool`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M56-02-edit-diff-flow.spec.md`
- Read: `docs/specs/M56-coding-agent-golden-path-v1.spec.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `crates/agent-tool/src/builtins/edit_file.rs`
- Read: `crates/agent-tool/src/builtins/git_diff.rs`
- Read: `crates/agent-tool/src/builtins/git_status.rs`
- Read: `crates/agent-core/src/tool_loop.rs`
- Read: `crates/agent-tui/src/diff_panel.rs`

- [x] **Step 1: Review edit/diff boundaries**

Run:

```bash
sed -n '1,260p' docs/specs/M56-02-edit-diff-flow.spec.md
```

Expected: scope is limited to edit preview/apply and diff evidence, not command/test execution or final review.

### Task 2: Write Failing Core Tests

**Files:**
- Create: `crates/agent-core/tests/edit_diff_flow.rs`

- [x] **Step 1: Add core tests**

Cover:

- preview-only returns diff and does not mutate file
- apply mutates the target file and reports changed files
- expected-content mismatch leaves file unchanged
- non-Git workspace succeeds with recoverable diff diagnostic
- traversal is rejected before mutation
- returned events include proposed, permission decision, execution, completion/failure, and observation evidence

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
cargo test -p agent-core edit_diff_flow
```

Expected: tests fail before the core flow exists.

### Task 3: Implement Core Edit/Diff Flow

**Files:**
- Create: `crates/agent-core/src/edit_diff.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [x] **Step 1: Add typed contracts**

Implement `EditDiffFlowInput`, `EditDiffFlowReport`, `EditPreviewSummary`, `DiffSummary`, diagnostics, and errors.

- [x] **Step 2: Orchestrate existing tools**

Use existing `FileEditTool`, `GitStatusTool`, and `GitDiffTool` through `ToolLoop` and policy. Do not hand-edit files directly from the flow.

- [x] **Step 3: Extract safe evidence**

Parse tool outputs into changed files, bounded diff preview, line counts, content hash/truncation flags, and diagnostics.

- [x] **Step 4: Run core tests**

Run:

```bash
cargo test -p agent-core edit_diff_flow
```

Expected: core edit/diff tests pass.

### Task 4: Add UI Bridge and TUI Evidence

**Files:**
- Create: `crates/agent-ui-bridge/tests/edit_diff_snapshot.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`
- Modify if needed: `crates/agent-tui/src/diff_panel.rs`
- Modify if needed: `crates/agent-tui/tests/diff_panel.rs`

- [x] **Step 1: Add bridge snapshot tests**

Cover safe serialization, truncation, redaction, and frame encoding.

- [x] **Step 2: Implement bridge snapshot**

Expose a UI-safe diff snapshot derived from `EditDiffFlowReport`.

- [x] **Step 3: Ensure TUI can consume evidence**

Prefer extending `DiffPanelState` from bridge snapshot while preserving existing approval-preview behavior.

- [x] **Step 4: Run bridge/TUI tests**

Run:

```bash
cargo test -p agent-ui-bridge edit_diff_snapshot
cargo test -p agent-tui diff_panel
```

Expected: UI evidence tests pass.

### Task 5: Add CLI Text and JSON Surface

**Files:**
- Create: `crates/agent-cli/tests/edit_diff_flow.rs`
- Modify: `crates/agent-cli/src/command.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- `repo edit --workspace . --path README.md --replacement-file path --preview`
- `repo edit --workspace . --path README.md --replacement-file path --apply`
- `--json repo edit ...` emits diff snapshot, events, and done frame
- invalid workspace/path exits with code `2`
- secret-like diff or diagnostics are redacted/bounded

- [x] **Step 2: Implement CLI command**

Add parser and renderer using core edit/diff flow. Prefer `--replacement-file` and `--expected-file`; literal flags may exist only for tests or small content.

- [x] **Step 3: Run CLI tests**

Run:

```bash
cargo test -p agent-cli edit_diff_flow
```

Expected: CLI tests pass.

### Task 6: Verify and Commit

**Files:**
- Modify: `docs/plans/M56-02-edit-diff-flow.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-core -p agent-cli -p agent-ui-bridge -p agent-tui --all-targets -- -D warnings
cargo test -p agent-tool file_edit_tool
cargo test -p agent-tool git_tools
cargo test -p agent-core edit_diff_flow
cargo test -p agent-cli edit_diff_flow
cargo test -p agent-ui-bridge edit_diff_snapshot
cargo test -p agent-tui diff_panel
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging for M56-02 source, tests, and docs, then:

```bash
git commit -m "feat: add m56 edit diff flow"
```

Expected: commit contains only M56-02 source, tests, and documentation/status updates.

### Completion Checklist

- [x] Core edit/diff flow exists and uses tool-loop/tool-policy boundaries.
- [x] Preview mode does not mutate files.
- [x] Apply mode reports changed files and diff evidence.
- [x] CLI text and JSON surfaces exist.
- [x] UI bridge/TUI diff evidence is safe and serializable.
- [x] Redaction/truncation/security cases are tested.
- [x] Focused fmt/clippy/tests pass.
