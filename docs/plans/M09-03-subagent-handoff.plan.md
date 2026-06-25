# M09-03 Subagent Handoff Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `agent-team` subagent profile, handoff, and capability reduction contracts.

**Architecture:** `agent-team` depends on `agent-types` and `agent-policy` only. This slice defines auditable delegation data; it does not spawn processes or execute tools.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `thiserror`, existing `agent-types`, `agent-policy`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold `agent-team`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/agent-team/Cargo.toml`
- Create: `crates/agent-team/src/lib.rs`
- Create: `crates/agent-team/src/error.rs`

- [ ] **Step 1: Add crate scaffold**

Add `crates/agent-team` to workspace members.

Create the crate with dependencies:

```toml
[dependencies]
agent-policy = { path = "../agent-policy" }
agent-types = { path = "../agent-types" }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

### Task 2: Write Handoff RED Tests

**Files:**
- Create: `crates/agent-team/tests/subagent_handoff.rs`

- [ ] **Step 1: Add failing tests**

Cover:

- `subagent_handoff_serializes_envelope_shape`
- `subagent_handoff_reduces_capabilities_by_intersection`
- `subagent_handoff_never_expands_parent_capabilities`
- `subagent_handoff_rejects_invalid_inputs`
- `subagent_handoff_rejects_terminal_status_transition`

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-team subagent_handoff
```

Expected: fails because `agent-team` contracts do not exist.

### Task 3: Implement Handoff Contracts

**Files:**
- Create: `crates/agent-team/src/capability.rs`
- Create: `crates/agent-team/src/handoff.rs`
- Modify: `crates/agent-team/src/lib.rs`

- [ ] **Step 1: Add capability contracts**

Implement `CapabilityGrant`, `CapabilitySet`, and `reduce_capabilities`.

- [ ] **Step 2: Add subagent handoff contracts**

Implement `SubagentRole`, `SubagentProfile`, `HandoffStatus`, `HandoffEnvelope`, and status transition validation.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-team subagent_handoff
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M09-03-subagent-handoff.plan.md`

- [ ] **Step 1: Advance active focus**

Set active focus to `M09-04-team-blackboard`.

- [ ] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m09 subagent handoff"
```
