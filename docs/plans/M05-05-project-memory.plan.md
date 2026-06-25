# M05-05 Project Memory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add project-scoped memory with audit entries and prove it can feed deterministic context assembly.

**Architecture:** Extend `agent-memory` with audit contracts and project-scope behavior. Add `agent-core` integration coverage that combines `agent-memory` records with `agent-context` prompt layers without putting memory logic into `agent-core`.

**Tech Stack:** Rust 2024 edition, `agent-memory`, `agent-context`, `agent-core`, serde, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Project Memory Tests

**Files:**
- Create: `crates/agent-memory/tests/project_memory.rs`

- [x] **Step 1: Add failing tests**

Add tests named:

- `project_memory_adds_lists_and_deletes_by_workspace`
- `project_memory_isolates_workspaces`
- `project_memory_records_add_and_delete_audit_entries`
- `project_memory_does_not_audit_failed_validation`

Use two `WorkspaceId` values and `MemoryScope::Project { workspace_id }`.

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-memory project_memory
```

Expected: tests fail because audit contracts do not exist.

### Task 2: Implement Project Memory Audit

**Files:**
- Create: `crates/agent-memory/src/audit.rs`
- Modify: `crates/agent-memory/src/store.rs`
- Modify: `crates/agent-memory/src/lib.rs`

- [x] **Step 1: Add audit contracts**

Define `MemoryAuditAction`, `MemoryAuditEntry`, and `AuditedMemoryStore`. Export them from `lib.rs`.

- [x] **Step 2: Record audit entries**

Extend `InMemoryMemoryStore` with `Vec<MemoryAuditEntry>`. Add an `Added` entry after successful `add`; add a `Deleted` entry after successful `delete`.

- [x] **Step 3: Verify project memory GREEN**

```bash
cargo test -p agent-memory project_memory
```

Expected: all project memory tests pass.

### Task 3: Add Core Context-Memory Integration

**Files:**
- Create: `crates/agent-core/tests/m05_context_memory_integration.rs`
- Modify: `crates/agent-core/Cargo.toml`

- [x] **Step 1: Add failing integration test**

Write `m05_context_memory_project_memory_feeds_prompt_bundle`. The test should add one project memory record, convert it to a `PromptLayer { kind: PromptLayerKind::Memory, priority: 20, ... }`, assemble a bundle, and assert the memory message appears before user content with stable token totals.

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-core m05_context_memory_project_memory_feeds_prompt_bundle
```

Expected: fails until `agent-core` dev-dependencies include `agent-context` and `agent-memory`.

- [x] **Step 3: Wire dev-dependencies and verify GREEN**

Add `agent-context` and `agent-memory` as `agent-core` dev-dependencies. Re-run the focused test.

```bash
cargo test -p agent-core m05_context_memory_project_memory_feeds_prompt_bundle
```

Expected: integration test passes.

### Task 4: M05 Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M05-05-project-memory.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Mark M05 complete**

Update `docs/milestones/README.md` so M05 implementation is complete and active focus advances to `M06-json-ui-bridge`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m05 context memory"
```
