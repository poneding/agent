# M10-04 Trace Timeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Desktop trace timeline contracts over `agent-core::TraceEntry`.

**Architecture:** `agent-core` remains responsible for querying and redacting runtime traces. `agent-desktop` converts trace entries into a deterministic timeline view model.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `agent-desktop`, `agent-core`, `agent-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M10-04-trace-timeline.spec.md`
- Read: `crates/agent-core/src/trace.rs`

- [x] **Step 1: Review the spec**

```bash
sed -n '1,260p' docs/specs/M10-04-trace-timeline.spec.md
```

### Task 2: Write Trace Timeline RED Tests

**Files:**
- Create: `crates/agent-desktop/tests/trace_timeline.rs`

- [x] **Step 1: Add failing tests**

Cover:

- `trace_timeline_serializes_timeline_shape`
- `trace_timeline_sorts_by_sequence`
- `trace_timeline_rejects_duplicate_sequences`
- `trace_timeline_rejects_empty_kind_and_summary`
- `trace_timeline_does_not_expose_secret_like_fields`

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-desktop trace_timeline
```

Expected: fails because trace timeline contracts do not exist.

### Task 3: Implement Trace Timeline Contracts

**Files:**
- Modify: `crates/agent-desktop/Cargo.toml`
- Create: `crates/agent-desktop/src/trace.rs`
- Modify: `crates/agent-desktop/src/lib.rs`
- Modify: `crates/agent-desktop/src/error.rs`

- [x] **Step 1: Add timeline structs**

Implement `TraceTimelineSeverity`, `TraceTimelineItem`, and `TraceTimeline`.

- [x] **Step 2: Add timeline builder**

Implement `build_trace_timeline` with sorting, validation, duplicate sequence rejection, and severity mapping.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p agent-desktop trace_timeline
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M10-04-trace-timeline.plan.md`

- [x] **Step 1: Advance active focus**

Set active focus to `M10-05-settings`.

- [x] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m10 trace timeline"
```
