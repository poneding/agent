# M23-05 Memory Editor and Portability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add memory tombstones, rebuild/import/export safeguards, UI-safe summaries, and CLI memory inspection/editing commands.

**Architecture:** `agent-memory` owns tombstone and portability semantics. `agent-cli` renders typed memory commands over memory runtime contracts. Durable platform paths remain in M25.

**Tech Stack:** Rust 2024 edition, serde, memory archive contracts, CLI JSON frames, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M23-05-memory-editor-portability.spec.md`
- Read: `docs/specs/M23-00-memory-retrieval-technical-design.spec.md`
- Read: `crates/agent-memory/src/archive.rs`
- Read: `crates/agent-cli/src/command.rs`

- [x] **Step 1: Review editor and portability scope**

Run:

```bash
sed -n '1,280p' docs/specs/M23-05-memory-editor-portability.spec.md
```

Expected: tombstone, rebuild, import/export, CLI, and redaction requirements are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-memory/tests/memory_editor_portability.rs`
- Create: `crates/agent-cli/tests/memory_commands.rs`

- [x] **Step 1: Add tombstone and rebuild tests**

Cover delete, repeated delete, rebuild skipping tombstones, and stale embedding cleanup.

- [x] **Step 2: Add import/export tests**

Cover tombstone preservation, conflict rejection/skipping, sensitive export behavior, and corrupted archive failure.

- [x] **Step 3: Add CLI command tests**

Cover `agent memory list|inspect|delete|export|import|rebuild-index` in text and JSON modes.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-memory memory_editor_portability
cargo test -p agent-cli memory_commands
```

Expected: tests fail because editor, tombstone, and CLI contracts are missing.

### Task 3: Implement Editor and Portability Contracts

**Files:**
- Create: `crates/agent-memory/src/tombstone.rs`
- Create: `crates/agent-memory/src/editor.rs`
- Modify: `crates/agent-memory/src/archive.rs`
- Modify: `crates/agent-memory/src/lib.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Modify if needed: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Add tombstone contracts**

Implement `MemoryTombstone`, tombstone store behavior, idempotent delete, and redacted delete reasons.

- [x] **Step 2: Add rebuild and portability behavior**

Implement `MemoryIndexRebuildReport`, import tombstone checks, export policy defaults, and stale embedding cleanup hooks.

- [x] **Step 3: Add memory summaries**

Implement UI-safe summaries with preview omission for sensitive memories and stable ordering.

- [x] **Step 4: Add CLI commands**

Parse and render memory list, inspect, delete, export, import, and rebuild-index commands in text/JSON modes.

### Task 4: Verify

**Files:**
- Read: `crates/agent-memory/tests/memory_editor_portability.rs`
- Read: `crates/agent-cli/tests/memory_commands.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-memory memory_editor_portability
cargo test -p agent-cli memory_commands
```

Expected: focused tests pass.

- [x] **Step 2: Run final M23 checks**

Run:

```bash
cargo xtask check
```

Expected: workspace checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M23-05-memory-editor-portability.plan.md`
- Modify: `docs/plans/M23-memory-retrieval-quality.plan.md`

- [x] **Step 1: Mark M23-05 and M23 complete**

Update milestone status and advance active focus to M24.

- [x] **Step 2: Commit M23 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m23 memory retrieval"
```

Expected: commit succeeds.
