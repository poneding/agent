# M22-03 Skill Root Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add registry-backed skill root management and diagnostics while keeping prompt contribution logic in `ruse-skill` and `ruse-context`.

**Architecture:** `ruse-skill` owns root validation and discovery diagnostics. `ruse-core` stores root extension state. `ruse-cli` renders skill root commands.

**Tech Stack:** Rust 2024 edition, tempfile fixtures, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M22-03-skill-root-management.spec.md`
- Read: `crates/ruse-skill/src/discovery.rs`
- Read: `crates/ruse-skill/tests/skill_discovery.rs`
- Read: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Review skill root scope**

Run:

```bash
sed -n '1,260p' docs/specs/M22-03-skill-root-management.spec.md
```

Expected: root commands, diagnostics, prompt boundaries, and redaction requirements are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-skill/tests/skill_roots.rs`
- Create: `crates/ruse-cli/tests/skill_root_commands.rs`

- [x] **Step 1: Add root model tests**

Cover add/list/enable/disable/remove, duplicate paths, missing paths, and deterministic root ids.

- [x] **Step 2: Add diagnostic tests**

Cover missing `SKILL.md`, invalid metadata, duplicate skill ids, empty prompt body, disabled root exclusion, and redaction.

- [x] **Step 3: Add CLI command tests**

Cover text and JSON output for root list, add, inspect, enable, disable, and remove.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p ruse-skill skill_roots
cargo test -p ruse-cli skill_root_commands
```

Expected: tests fail because managed skill roots and commands are missing.

### Task 3: Implement Skill Root Management

**Files:**
- Create: `crates/ruse-skill/src/roots.rs`
- Modify: `crates/ruse-skill/src/lib.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add managed root contracts**

Implement `ManagedSkillRoot`, `SkillRootInspection`, `DiscoveredSkillSummary`, and `SkillRootDiagnostic`.

- [x] **Step 2: Add root operations**

Implement add/list/inspect/enable/disable/remove with deterministic ordering and validation.

- [x] **Step 3: Connect discovery diagnostics**

Convert existing discovery errors into root diagnostics without printing full prompt bodies.

- [x] **Step 4: Add CLI renderers**

Render stable text and JSON outputs for skill root commands.

### Task 4: Verify

**Files:**
- Read: `crates/ruse-skill/tests/skill_roots.rs`
- Read: `crates/ruse-cli/tests/skill_root_commands.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-skill skill_roots
cargo test -p ruse-cli skill_root_commands
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `docs/plans/M22-03-skill-root-management.plan.md`
- Modify: `docs/plans/M22-extension-ux-mcp-gateway.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M22-03 complete**

Advance active implementation focus to M22-04.

- [x] **Step 2: Commit M22-03**

Run:

```bash
git add crates/ruse-skill crates/ruse-cli docs Cargo.lock
git commit -m "feat: add skill root management"
```

Expected: commit succeeds.
