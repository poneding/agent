# M41-04 Sampling Bridge Guardrails Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic guardrails for unsupported MCP sampling model bridge paths.

**Architecture:** Reuse `ruse-mcp` validation before model streaming and reject unsupported model stream events inside `ruse-core::mcp_sampling`.

**Tech Stack:** Rust 2024 edition, ruse-core, ruse-model fake provider, Cargo tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M41-04-sampling-bridge-guardrails.spec.md`
- Read: `crates/ruse-core/src/mcp_sampling.rs`
- Read: `crates/ruse-model/src/fake.rs`

- [x] **Step 1: Review unsupported paths**

Run:

```bash
sed -n '1,260p' docs/specs/M41-04-sampling-bridge-guardrails.spec.md
sed -n '1,180p' crates/ruse-model/src/fake.rs
```

Expected: fake provider can emit model ToolCall events for rejection tests.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-core/tests/mcp_sampling_model_guardrails.rs`

- [x] **Step 1: Add guardrail tests**

Add tests named:

```rust
mcp_sampling_model_guardrails_reject_tools_before_streaming
mcp_sampling_model_guardrails_reject_context_before_streaming
mcp_sampling_model_guardrails_reject_model_tool_call_without_input_leak
mcp_sampling_model_guardrails_reject_media_before_streaming
```

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo test -p ruse-core mcp_sampling_model_guardrails
```

Expected: tests fail until guardrails are explicit and covered.

### Task 3: Implement Guardrails

**Files:**
- Modify: `crates/ruse-core/src/mcp_sampling.rs`

- [x] **Step 1: Reject invalid requests before provider execution**

Ensure validation happens before calling `ModelProvider::stream`.

- [x] **Step 2: Reject model tool-call stream events**

Return a redacted bridge error when `ModelStreamEvent::ToolCall` appears.

- [x] **Step 3: Keep media rejection before streaming**

Ensure image/audio content produces unsupported content errors before provider execution.

### Task 4: Verify

- [x] **Step 1: Run focused checks**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m41 cargo clippy -p ruse-core --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m41 cargo test -p ruse-core mcp_sampling_model_guardrails
```

Expected: focused checks pass.

### Task 5: Commit

- [x] **Step 1: Commit M41-04**

Run:

```bash
git add crates/ruse-core docs
git commit -m "feat: guard mcp sampling model bridge paths"
```
