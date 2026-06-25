# Mxx-yy Topic Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** State the concrete implementation outcome.

**Architecture:** Name the crates and the dependency direction. Keep runtime logic out of UI crates and keep durable contracts typed.

**Tech Stack:** Rust 2024 edition, Cargo workspace, relevant crate dependencies, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/Mxx-yy-topic.spec.md`
- Read: `docs/architecture/README.md`

- [ ] **Step 1: Review the spec**

Run:

```bash
sed -n '1,260p' docs/specs/Mxx-yy-topic.spec.md
```

Expected: the spec defines scope, contracts, tests, and acceptance criteria.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/crate-name/tests/topic.rs`

- [ ] **Step 1: Add focused tests**

```rust
#[test]
fn behavior_name_matches_contract() {
    let actual = "contract";
    assert_eq!(actual, "contract");
}
```

- [ ] **Step 2: Run focused tests and verify failure**

Run:

```bash
cargo test -p crate-name behavior_name_matches_contract
```

Expected: test fails before the implementation exists.

### Task 3: Implement Minimal Contract

**Files:**
- Modify: `crates/crate-name/src/lib.rs`

- [ ] **Step 1: Add implementation**

```rust
#[must_use]
pub const fn contract_name() -> &'static str {
    "contract"
}
```

- [ ] **Step 2: Run focused tests and verify success**

Run:

```bash
cargo test -p crate-name behavior_name_matches_contract
```

Expected: focused tests pass.

### Task 4: Run Verification

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [ ] **Step 1: Run standard checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Commit

**Files:**
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Update progress and commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete mxx-yy topic"
```

Expected: commit succeeds with source, tests, and documentation together.
