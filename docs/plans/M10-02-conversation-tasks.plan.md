# M10-02 Conversation and Tasks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Desktop conversation, task, and workspace snapshot view-model contracts.

**Architecture:** Implement view models inside `agent-desktop`. Keep the contracts UI-safe and independent of CLI formatting, storage rows, and model provider types.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `agent-desktop`, `agent-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M10-02-conversation-tasks.spec.md`
- Read: `crates/agent-desktop/src/lib.rs`

- [ ] **Step 1: Review the spec**

```bash
sed -n '1,260p' docs/specs/M10-02-conversation-tasks.spec.md
```

### Task 2: Write Conversation and Task RED Tests

**Files:**
- Create: `crates/agent-desktop/tests/conversation_tasks.rs`

- [ ] **Step 1: Add failing tests**

Cover:

- `conversation_tasks_serializes_snapshot_shape`
- `conversation_tasks_sort_conversations_by_recent_activity`
- `conversation_tasks_sort_active_tasks_before_terminal_tasks`
- `conversation_tasks_reject_empty_titles_and_summaries`
- `conversation_tasks_snapshot_excludes_raw_prompt_and_tool_input`

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-desktop conversation_tasks
```

Expected: fails because conversation/task contracts do not exist.

### Task 3: Implement Conversation and Task Contracts

**Files:**
- Create: `crates/agent-desktop/src/conversation.rs`
- Create: `crates/agent-desktop/src/task.rs`
- Create: `crates/agent-desktop/src/snapshot.rs`
- Modify: `crates/agent-desktop/src/lib.rs`
- Modify: `crates/agent-desktop/src/error.rs`

- [ ] **Step 1: Add view-model structs**

Implement `DesktopConversationItem`, `DesktopTaskItem`, and `DesktopWorkspaceSnapshot`.

- [ ] **Step 2: Add snapshot builder**

Implement `build_workspace_snapshot` with validation and deterministic sorting.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-desktop conversation_tasks
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M10-02-conversation-tasks.plan.md`

- [ ] **Step 1: Advance active focus**

Set active focus to `M10-03-permissions-diffs`.

- [ ] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m10 conversation tasks"
```
