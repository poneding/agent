# M09-04 Team Blackboard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add team blackboard contracts and an in-memory store for subagent findings, artifact references, reviews, and trace links.

**Architecture:** Blackboard behavior lives in `agent-team` and builds on the crate introduced in M09-03. It stores references and safe summaries, not raw artifacts.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `agent-team`, `agent-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M09-04-team-blackboard.spec.md`
- Read: `crates/agent-team/src/lib.rs`

- [x] **Step 1: Review the spec**

```bash
sed -n '1,260p' docs/specs/M09-04-team-blackboard.spec.md
```

### Task 2: Write Blackboard RED Tests

**Files:**
- Create: `crates/agent-team/tests/team_blackboard.rs`

- [x] **Step 1: Add failing tests**

Cover:

- `team_blackboard_serializes_entry_shape`
- `team_blackboard_appends_and_lists_entries_in_order`
- `team_blackboard_rejects_empty_summary`
- `team_blackboard_rejects_unsafe_artifact_path`
- `team_blackboard_preserves_review_vote_values`

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-team team_blackboard
```

Expected: fails because blackboard APIs do not exist.

### Task 3: Implement Blackboard Contracts

**Files:**
- Create: `crates/agent-team/src/blackboard.rs`
- Modify: `crates/agent-team/src/lib.rs`

- [x] **Step 1: Add blackboard data contracts**

Implement `BlackboardEntryKind`, `ArtifactRef`, `ReviewVote`, and `BlackboardEntry`.

- [x] **Step 2: Add in-memory store**

Implement `BlackboardStore` and `InMemoryBlackboardStore`.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p agent-team team_blackboard
```

### Task 4: Complete M09

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M09-04-team-blackboard.plan.md`

- [x] **Step 1: Mark M09 complete**

Set active focus to M10 planning.

- [x] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m09 worktree subagents"
```
