# M19-03 Tool Preview Payloads Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate typed, secret-safe previews for side-effecting tools before approval.

**Architecture:** `ruse-tool` owns preview payloads because it owns built-in tool schemas and workspace path handling. `ruse-policy` stores only the summary projection from those previews.

**Tech Stack:** Rust 2024 edition, serde, existing file edit/shell/Git tool modules, deterministic diff tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M19-03-tool-preview-payloads.spec.md`
- Read: `crates/ruse-tool/src/builtins/edit_file.rs`
- Read: `crates/ruse-tool/src/builtins/shell_command.rs`
- Read: `crates/ruse-tool/src/builtins/git_status.rs`
- Read: `crates/ruse-tool/src/builtins/git_diff.rs`

- [x] **Step 1: Review preview scope**

Run:

```bash
sed -n '1,220p' docs/specs/M19-03-tool-preview-payloads.spec.md
rg -n "preview|diff|execute" crates/ruse-tool/src
```

Expected: preview generation is side-effect free and UI-safe.

### Task 2: Write RED Preview Tests

**Files:**
- Create: `crates/ruse-tool/tests/tool_previews.rs`

- [x] **Step 1: Add preview tests**

Add tests for file edit diff summaries, shell command summaries, Git summaries, generic MCP/plugin summaries, redaction, truncation, and unsafe path rejection.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-tool --test tool_previews
```

Expected: compile fails because preview contracts do not exist.

### Task 3: Implement Preview Contracts

**Files:**
- Create: `crates/ruse-tool/src/preview.rs`
- Modify: `crates/ruse-tool/src/lib.rs`

- [x] **Step 1: Add `ToolPreview` enum and payload structs**

Implement file edit, command, Git, MCP, plugin, background task, and generic preview payloads.

- [x] **Step 2: Add redaction and truncation helpers**

Provide deterministic helpers for secret-like keys, env allowlists, byte limits, and line limits.

- [x] **Step 3: Add summary conversion**

Convert `ToolPreview` into `ruse_policy::ApprovalPreviewSummary`.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-tool/tests/tool_previews.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-tool --test tool_previews
```

Expected: focused tests pass.

- [x] **Step 2: Run tool crate tests**

Run:

```bash
cargo test -p ruse-tool
```

Expected: all tool tests pass.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M19-03-tool-preview-payloads.plan.md`
- Modify: `docs/plans/M19-interactive-approval-safe-actions.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M19-03 complete**

Update plan and milestone checkboxes after verification.
