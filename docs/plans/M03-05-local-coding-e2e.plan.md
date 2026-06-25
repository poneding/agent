# M03-05 Local Coding E2E Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect CLI, policy, tools, storage, and audit events into a deterministic local coding workflow.

**Architecture:** `agent-core` owns a fixture-friendly local workflow runner. `agent-cli` exposes a smoke command that renders the outcome while tool execution remains policy-gated and auditable.

**Tech Stack:** Rust 2024 edition, M03 runtime crates, `serde_json`, `tempfile`, local `git`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Core E2E Tests

**Files:**
- Create: `crates/agent-core/tests/local_coding_e2e.rs`

- [ ] **Step 1: Add failing E2E tests**

Tests must prove read/edit/shell/Git workflow, denied command branch, changed files summary, command results summary, and audit event count.

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-core local_coding_e2e
```

Expected: tests fail because local coding workflow does not exist.

### Task 2: Implement Local Coding Workflow

**Files:**
- Create: `crates/agent-core/src/local_coding.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [ ] **Step 1: Define outcome**

Implement `LocalCodingOutcome` with final answer, changed files, command results, and audit event count.

- [ ] **Step 2: Implement deterministic workflow runner**

Use existing tool, policy, storage, and audit contracts to execute the fixture workflow.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-core local_coding_e2e
```

Expected: focused E2E tests pass.

### Task 3: Add CLI Local Coding Smoke

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/src/main.rs`
- Create: `crates/agent-cli/tests/local_coding.rs`

- [ ] **Step 1: Add failing CLI smoke test**

Test `agent-cli --local-summary` prints final answer, changed files, command results, and audit summary.

- [ ] **Step 2: Implement CLI adapter**

Render `LocalCodingOutcome` from the fixture runner.

- [ ] **Step 3: Verify focused CLI test**

```bash
cargo test -p agent-cli local_coding
```

Expected: CLI local coding test passes.

### Task 4: Run M03 Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M03-05-local-coding-e2e.plan.md`

- [ ] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 2: Mark M03 complete**

Update milestone status so M03 is complete and active focus advances to `M04-checkpoint-recovery-trace`.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m03 local coding loop"
```
