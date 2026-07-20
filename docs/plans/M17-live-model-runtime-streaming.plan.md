# M17 Live Model Runtime and Streaming Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add live OpenAI-compatible HTTP model execution and true runtime streaming while preserving provider-neutral core events.

**Architecture:** `ruse-model` owns streaming provider traits, reqwest transport, SSE parsing, and secret-safe errors. `ruse-core` consumes model streams incrementally. `ruse-cli` registers live providers from config and renders streamed text or NDJSON.

**Tech Stack:** Rust 2024 edition, Tokio, reqwest, futures stream traits, serde, fixture HTTP streams, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M17 Technical Design

**Files:**
- Create: `docs/specs/M17-00-live-model-runtime-technical-design.spec.md`
- Create: `docs/plans/M17-00-live-model-runtime-technical-design.plan.md`

- [x] **Step 1: Write the M17 technical design baseline**

Define stream trait migration, reqwest transport, secret injection, retry/fallback behavior, cancellation, fixture testing, and CLI rendering.

- [x] **Step 2: Write the M17 design execution plan**

Document implementation order, compatibility boundaries, and migration strategy from collected vectors to streaming events.

### Task 2: Decompose M17 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M17-01-model-stream-trait.spec.md`
- Create: `docs/plans/M17-01-model-stream-trait.plan.md`
- Create: `docs/specs/M17-02-openai-reqwest-transport.spec.md`
- Create: `docs/plans/M17-02-openai-reqwest-transport.plan.md`
- Create: `docs/specs/M17-03-live-provider-registration.spec.md`
- Create: `docs/plans/M17-03-live-provider-registration.plan.md`
- Create: `docs/specs/M17-04-cli-streaming-output.spec.md`
- Create: `docs/plans/M17-04-cli-streaming-output.plan.md`
- Create: `docs/specs/M17-05-streaming-retry-cancel-replay.spec.md`
- Create: `docs/plans/M17-05-streaming-retry-cancel-replay.plan.md`

- [x] **Step 1: Create slice specs**

Each spec defines public stream contracts, fixture strategy, privacy rules, and acceptance criteria.

- [x] **Step 2: Create slice plans**

Each plan starts with failing stream-focused tests and includes workspace verification.

### Task 3: Implement M17 Slices in Order

**Files:**
- Execute: `docs/plans/M17-01-model-stream-trait.plan.md`
- Execute: `docs/plans/M17-02-openai-reqwest-transport.plan.md`
- Execute: `docs/plans/M17-03-live-provider-registration.plan.md`
- Execute: `docs/plans/M17-04-cli-streaming-output.plan.md`
- Execute: `docs/plans/M17-05-streaming-retry-cancel-replay.plan.md`

- [x] **Step 1: Migrate model provider stream contracts**

Replace collected model results with true async stream consumption while preserving fixture compatibility.

- [x] **Step 2: Implement OpenAI-compatible reqwest transport**

Add safe header injection, SSE body streaming, status classification, and redacted errors.

- [x] **Step 3: Register live providers from config**

Wire secret resolution and provider registration in CLI without moving provider-specific code into core.

- [x] **Step 4: Render streamed CLI output**

Emit text deltas and JSON frames as events arrive.

- [x] **Step 5: Add retry, cancellation, and replay coverage**

Ensure streaming attempts remain traceable and recoverable.

### Task 4: Complete M17

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M17-live-model-runtime-streaming.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all workspace gates pass without requiring live network access.

- [x] **Step 2: Mark M17 complete**

Update milestone status and advance active focus to M18.

- [ ] **Step 3: Commit M17 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m17 live model runtime"
```

Expected: commit succeeds.
