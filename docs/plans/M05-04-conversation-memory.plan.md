# M05-04 Conversation Memory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `ruse-memory` with conversation-scoped memory records and an in-memory store.

**Architecture:** `ruse-memory` owns memory records, validation, and scoped retrieval. It depends on `ruse-types`, `serde`, `serde_json`, and `thiserror`; it does not depend on `ruse-context` or `ruse-core`.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold `ruse-memory`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-memory/Cargo.toml`
- Create: `crates/ruse-memory/src/lib.rs`
- Create: `crates/ruse-memory/src/error.rs`
- Create: `crates/ruse-memory/src/record.rs`
- Create: `crates/ruse-memory/src/store.rs`

- [x] **Step 1: Add failing crate smoke test**

Create `crates/ruse-memory/tests/conversation_memory.rs` with:

```rust
use ruse_memory::MemorySensitivity;

#[test]
fn memory_sensitivity_uses_snake_case_json() {
    assert_eq!(serde_json::to_string(&MemorySensitivity::Internal).unwrap(), "\"internal\"");
}
```

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-memory memory_sensitivity_uses_snake_case_json
```

Expected: fails because `ruse-memory` is not a workspace crate yet.

- [x] **Step 3: Add crate scaffold**

Add `crates/ruse-memory` to workspace members. Create crate dependencies on `ruse-types`, `serde`, `serde_json`, and `thiserror`. Export `MemoryError`, `MemoryRecord`, `MemoryScope`, `MemorySensitivity`, `MemorySource`, `MemoryStore`, and `InMemoryMemoryStore`.

- [x] **Step 4: Verify crate smoke GREEN**

```bash
cargo test -p ruse-memory memory_sensitivity_uses_snake_case_json
```

Expected: smoke test passes.

### Task 2: Write Conversation Memory Tests

**Files:**
- Modify: `crates/ruse-memory/tests/conversation_memory.rs`

- [x] **Step 1: Add failing behavior tests**

Add tests named:

- `conversation_memory_adds_and_lists_records_by_scope`
- `conversation_memory_isolates_conversations`
- `conversation_memory_deletes_records`
- `conversation_memory_rejects_invalid_records`
- `memory_record_serializes_stable_shape`

Use two conversation IDs and records with confidence `80`.

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-memory conversation_memory
```

Expected: behavior tests fail because store contracts are not implemented.

### Task 3: Implement Conversation Memory Store

**Files:**
- Modify: `crates/ruse-memory/src/error.rs`
- Modify: `crates/ruse-memory/src/record.rs`
- Modify: `crates/ruse-memory/src/store.rs`
- Modify: `crates/ruse-memory/src/lib.rs`

- [x] **Step 1: Implement memory contracts**

Define memory enums and structs from the spec. Derive serde traits for durable public data.

- [x] **Step 2: Implement validation**

Reject empty IDs, empty text, empty source descriptions, and confidence outside `1..=100`.

- [x] **Step 3: Implement in-memory scoped store**

Use `BTreeMap<MemoryScope, BTreeMap<String, MemoryRecord>>` so list order is deterministic by record ID.

- [x] **Step 4: Verify GREEN**

```bash
cargo test -p ruse-memory --test conversation_memory
```

Expected: all conversation memory tests pass.

### Task 4: Workspace Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M05-04-conversation-memory.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M05-04-conversation-memory` to `M05-05-project-memory`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m05 conversation memory"
```
