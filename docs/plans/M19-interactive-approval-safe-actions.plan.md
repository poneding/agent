# M19 Interactive Approval and Safe Local Actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add durable interactive approval flows for side-effecting tools and resume turns after approval or denial.

**Architecture:** `agent-policy` owns leases and decisions, `agent-core` owns suspended turn state, `agent-ui-bridge` carries approval commands/events, and `agent-cli` implements the first interactive approval prompt.

**Tech Stack:** Rust 2024 edition, serde UI schemas, event storage, CLI terminal prompts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M19 Technical Design

**Files:**
- Create: `docs/specs/M19-00-interactive-approval-technical-design.spec.md`
- Create: `docs/plans/M19-00-interactive-approval-technical-design.plan.md`

- [x] **Step 1: Write the M19 technical design baseline**

Define pending approval storage, response commands, lease scope, preview payloads, resume tokens, timeout, and audit behavior.

- [x] **Step 2: Write the M19 design execution plan**

Document slice order and UI bridge compatibility requirements.

### Task 2: Decompose M19 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M19-01-pending-approval-contracts.spec.md`
- Create: `docs/plans/M19-01-pending-approval-contracts.plan.md`
- Create: `docs/specs/M19-02-ui-approval-bridge.spec.md`
- Create: `docs/plans/M19-02-ui-approval-bridge.plan.md`
- Create: `docs/specs/M19-03-tool-preview-payloads.spec.md`
- Create: `docs/plans/M19-03-tool-preview-payloads.plan.md`
- Create: `docs/specs/M19-04-cli-approval-prompt.spec.md`
- Create: `docs/plans/M19-04-cli-approval-prompt.plan.md`
- Create: `docs/specs/M19-05-approval-resume-recovery.spec.md`
- Create: `docs/plans/M19-05-approval-resume-recovery.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines typed events, commands, storage behavior, and security constraints.

- [x] **Step 2: Create slice plans**

Each plan starts with approval state tests and bridge snapshot tests.

### Task 3: Implement M19 Slices in Order

**Files:**
- Execute: `docs/plans/M19-01-pending-approval-contracts.plan.md`
- Execute: `docs/plans/M19-02-ui-approval-bridge.plan.md`
- Execute: `docs/plans/M19-03-tool-preview-payloads.plan.md`
- Execute: `docs/plans/M19-04-cli-approval-prompt.plan.md`
- Execute: `docs/plans/M19-05-approval-resume-recovery.plan.md`

- [x] **Step 1: Implement pending approval contracts**

Add durable pending approval, response, lease, and resume token contracts.

- [x] **Step 2: Add UI bridge approval schema**

Expose approval request and response frames shared by CLI, TUI, and Desktop.

- [x] **Step 3: Add tool preview payloads**

Create file diff, shell command, Git, MCP, and plugin preview summaries.

- [x] **Step 4: Implement CLI approval prompts**

Allow users to approve or deny side-effecting actions in terminal flows.

- [x] **Step 5: Implement resume and recovery**

Resume suspended turns after approval, denial, timeout, or crash recovery.

### Task 4: Complete M19

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M19-interactive-approval-safe-actions.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [x] **Step 2: Mark M19 complete**

Update milestone status and advance active focus to M20.

- [ ] **Step 3: Commit M19 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m19 interactive approvals"
```

Expected: commit succeeds.
