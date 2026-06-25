# M05-03 Pinning and Compact Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic pin preservation and rolling compaction contracts to `agent-context`.

**Architecture:** Extend `agent-context` with a `compact` module that works on `PromptMessage` values from M05-01. The first compactor is deterministic and does not call models or storage.

**Tech Stack:** Rust 2024 edition, `agent-context`, serde, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Pinning and Compaction Tests

**Files:**
- Create: `crates/agent-context/tests/pinning_compact.rs`

- [ ] **Step 1: Add failing tests**

Add tests named:

- `compact_messages_preserves_pinned_messages`
- `compact_messages_produces_stable_summary`
- `compact_messages_rejects_duplicate_pin_ids`
- `compact_messages_rejects_zero_summary_budget_for_unpinned_messages`

Use two pinned messages and three unpinned messages. Assert the pinned IDs and messages are unchanged and summary text is stable across two calls.

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-context pinning_compact
```

Expected: tests fail because compaction contracts do not exist.

### Task 2: Implement Compaction Contracts

**Files:**
- Create: `crates/agent-context/src/compact.rs`
- Modify: `crates/agent-context/src/error.rs`
- Modify: `crates/agent-context/src/lib.rs`

- [ ] **Step 1: Add public compact types**

Define `PinnedMessage`, `CompactionSummary`, `CompactRequest`, and `CompactResult`. Export them from `lib.rs`.

- [ ] **Step 2: Add compaction errors**

Extend `ContextError` with `InvalidPin(String)` and `InvalidCompaction(String)`.

- [ ] **Step 3: Verify type-level GREEN progress**

```bash
cargo test -p agent-context compact_messages_rejects_duplicate_pin_ids
```

Expected: duplicate pin test fails with the expected variant once validation is wired; other tests may still fail.

### Task 3: Implement Rolling Compactor

**Files:**
- Modify: `crates/agent-context/src/compact.rs`

- [ ] **Step 1: Validate pins**

Reject empty pin IDs, empty reasons, and duplicate pin IDs. Keep original pin order.

- [ ] **Step 2: Build deterministic summary**

For unpinned messages, join snippets as `"{role}: {content}"` separated by `"\n"`. Truncate by estimated token count using the existing `PromptMessage` content words as the approximation.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-context pinning_compact
```

Expected: all pinning and compaction tests pass.

### Task 4: Workspace Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M05-03-pinning-compact.plan.md`
- Modify: `README.md`

- [ ] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 2: Advance active focus**

Update active focus from `M05-03-pinning-compact` to `M05-04-conversation-memory`.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m05 pinning compact"
```
