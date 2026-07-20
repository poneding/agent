# M18 Agent Tool Loop Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the comprehensive model-tool-observation loop that powers real coding-agent turns.

**Architecture:** `ruse-core` owns loop orchestration. It consumes `ruse-model` stream events, executes `ruse-tool` calls through `ruse-policy`, records events in `ruse-storage`, and uses `ruse-context` to add compressed observations back to model input.

**Tech Stack:** Rust 2024 edition, Tokio, serde, existing tool/policy/storage crates, deterministic fake models, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M18 Technical Design

**Files:**
- Create: `docs/specs/M18-00-ruse-tool-loop-technical-design.spec.md`
- Create: `docs/plans/M18-00-ruse-tool-loop-technical-design.plan.md`

- [x] **Step 1: Write the M18 technical design baseline**

Define loop states, iteration guards, tool observation messages, idempotency, compaction, approval pauses, and recovery behavior.

- [x] **Step 2: Write the M18 design execution plan**

Document slice ordering and compatibility with existing `ToolLoop` tests.

### Task 2: Decompose M18 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M18-01-comprehensive-turn-runner.spec.md`
- Create: `docs/plans/M18-01-comprehensive-turn-runner.plan.md`
- Create: `docs/specs/M18-02-streaming-tool-call-accumulator.spec.md`
- Create: `docs/plans/M18-02-streaming-tool-call-accumulator.plan.md`
- Create: `docs/specs/M18-03-tool-observation-context.spec.md`
- Create: `docs/plans/M18-03-tool-observation-context.plan.md`
- Create: `docs/specs/M18-04-loop-guards-idempotency.spec.md`
- Create: `docs/plans/M18-04-loop-guards-idempotency.plan.md`
- Create: `docs/specs/M18-05-local-coding-real-loop-e2e.spec.md`
- Create: `docs/plans/M18-05-local-coding-real-loop-e2e.plan.md`

- [x] **Step 1: Create slice specs**

Each slice spec identifies state transitions, failure behavior, security controls, and replay requirements.

- [x] **Step 2: Create slice plans**

Each plan uses TDD with fake model scripts that propose tool calls.

### Task 3: Implement M18 Slices in Order

**Files:**
- Execute: `docs/plans/M18-01-comprehensive-turn-runner.plan.md`
- Execute: `docs/plans/M18-02-streaming-tool-call-accumulator.plan.md`
- Execute: `docs/plans/M18-03-tool-observation-context.plan.md`
- Execute: `docs/plans/M18-04-loop-guards-idempotency.plan.md`
- Execute: `docs/plans/M18-05-local-coding-real-loop-e2e.plan.md`

- [x] **Step 1: Implement comprehensive turn runner**

Add model/tool loop state orchestration behind a new core API.

- [x] **Step 2: Accumulate streaming tool calls**

Normalize full and partial tool call proposals into executable requests.

- [x] **Step 3: Feed observations back into context**

Compress and source-label tool results before the next model call.

- [x] **Step 4: Enforce guards and idempotency**

Apply iteration, tool, wall-time, output-size, and replay protections.

- [x] **Step 5: Prove local coding e2e**

Run a deterministic model-driven edit, shell, Git, and final response workflow.

### Task 4: Complete M18

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M18-ruse-tool-loop-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all gates pass.

- [x] **Step 2: Mark M18 complete**

Update milestone status and advance active focus to M19.

- [ ] **Step 3: Commit M18 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m18 agent tool loop"
```

Expected: commit succeeds.
