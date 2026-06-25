# M05-02 Budget Planner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic token budget planning to `agent-context`.

**Architecture:** Extend `agent-context` with a `budget` module that consumes `PromptLayer` values from M05-01. Keep the planner pure and independent of model providers.

**Tech Stack:** Rust 2024 edition, `agent-context`, serde, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Budget Planner Tests

**Files:**
- Create: `crates/agent-context/tests/budget_planner.rs`

- [x] **Step 1: Add failing tests**

Add tests named:

- `budget_planner_includes_required_and_fitting_optional_layers`
- `budget_planner_rejects_required_overflow`
- `budget_planner_omits_optional_layers_with_reasons`
- `context_budget_serializes_stable_shape`

Use three layers with token estimates `40`, `30`, and `50`; set `ContextBudget { max_tokens: 100, reserved_response_tokens: 20 }` so the first two fit and the third is omitted.

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-context budget_planner
```

Expected: tests fail because budget contracts do not exist.

### Task 2: Implement Budget Contracts

**Files:**
- Create: `crates/agent-context/src/budget.rs`
- Modify: `crates/agent-context/src/error.rs`
- Modify: `crates/agent-context/src/lib.rs`

- [x] **Step 1: Add public budget types**

Define `ContextBudget`, `BudgetClass`, `BudgetItem`, `BudgetDecision`, and `BudgetPlan`. Export them from `lib.rs`.

- [x] **Step 2: Add budget error variants**

Extend `ContextError` with `InvalidBudget(String)` and `BudgetExceeded(String)`.

- [x] **Step 3: Verify type-level GREEN progress**

```bash
cargo test -p agent-context context_budget_serializes_stable_shape
```

Expected: serialization test passes while behavior tests may still fail.

### Task 3: Implement Budget Planning

**Files:**
- Modify: `crates/agent-context/src/budget.rs`

- [x] **Step 1: Implement capacity calculation**

Return `InvalidBudget("reserved response tokens must be less than max tokens")` when `reserved_response_tokens >= max_tokens`.

- [x] **Step 2: Implement inclusion and omission**

Iterate items in caller order. Include required items if they fit; fail if a required item would exceed capacity. Include optional items only if they fit; otherwise push `(layer, "budget_exceeded")` to `omitted`.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p agent-context --test budget_planner
```

Expected: all budget planner tests pass.

### Task 4: Workspace Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M05-02-budget-planner.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M05-02-budget-planner` to `M05-03-pinning-compact`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m05 context budget planner"
```
