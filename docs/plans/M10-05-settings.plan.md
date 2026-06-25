# M10-05 Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Desktop settings contracts and complete the M10 Desktop Preview milestone.

**Architecture:** Keep settings as validated UI-facing snapshots inside `agent-desktop`. Do not introduce durable config files, migrations, or secret storage in this slice.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `agent-desktop`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M10-05-settings.spec.md`
- Read: `docs/architecture/configuration.md`

- [ ] **Step 1: Review the spec**

```bash
sed -n '1,260p' docs/specs/M10-05-settings.spec.md
```

### Task 2: Write Settings RED Tests

**Files:**
- Create: `crates/agent-desktop/tests/settings.rs`

- [ ] **Step 1: Add failing tests**

Cover:

- `settings_serializes_stable_shape`
- `settings_defaults_to_conservative_policy`
- `settings_rejects_empty_provider_and_model`
- `settings_rejects_invalid_policy_decision`
- `settings_serialized_shape_excludes_secret_like_keys`

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-desktop settings
```

Expected: fails because settings contracts do not exist.

### Task 3: Implement Settings Contracts

**Files:**
- Create: `crates/agent-desktop/src/settings.rs`
- Modify: `crates/agent-desktop/src/lib.rs`
- Modify: `crates/agent-desktop/src/error.rs`

- [ ] **Step 1: Add settings structs**

Implement `DesktopSettings`, `ModelSettings`, `PolicySettings`, `MemorySettings`, `McpSettings`, `DesktopTheme`, and `UiSettings`.

- [ ] **Step 2: Add defaults and validation**

Implement conservative defaults and settings validation.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-desktop settings
```

### Task 4: Complete M10

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M10-05-settings.plan.md`

- [ ] **Step 1: Mark M10 complete**

Set active focus to M11 planning.

- [ ] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m10 desktop preview"
```
