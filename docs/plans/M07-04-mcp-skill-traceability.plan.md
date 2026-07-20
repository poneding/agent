# M07-04 MCP and Skill Traceability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add trace-safe runtime and UI event contracts for MCP and skill source metadata.

**Architecture:** Runtime trace payloads live in `ruse-types`; UI-safe summaries live in `ruse-ui-bridge`. The slice avoids raw MCP schemas, raw tool input/output, raw skill prompt bodies, and secret values in trace payloads.

**Tech Stack:** Rust 2024 edition, Cargo workspace, existing `ruse-types`, `ruse-ui-bridge`, `ruse-core` tests, `serde`, `serde_json`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M07-04-mcp-skill-traceability.spec.md`
- Read: `crates/ruse-types/src/event.rs`
- Read: `crates/ruse-ui-bridge/src/event.rs`
- Read: `crates/ruse-core/tests`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,280p' docs/specs/M07-04-mcp-skill-traceability.spec.md
```

Expected: the spec defines trace payloads, serialization requirements, UI normalization, and security exclusions.

### Task 2: Write Traceability RED Tests

**Files:**
- Create: `crates/ruse-types/tests/m07_external_trace.rs`
- Create: `crates/ruse-ui-bridge/tests/m07_traceability.rs`
- Create: `crates/ruse-core/tests/m07_mcp_skill_traceability.rs`

- [x] **Step 1: Add ruse-types serialization tests**

Cover:

- `m07_external_trace_serializes_mcp_tools_discovered`
- `m07_external_trace_serializes_skill_prompt_contribution_without_body`
- `m07_external_trace_serializes_mcp_policy_mapping`

- [x] **Step 2: Add UI bridge normalization tests**

Cover:

- MCP tool discovery normalizes to a UI-safe payload or progress summary.
- Skill prompt contribution normalizes without the prompt body.
- MCP policy mapping normalizes with action and resource metadata.

- [x] **Step 3: Add core fixture test**

Build deterministic `RuntimeEvent` fixtures that include MCP and skill trace payloads and assert they serialize through the existing runtime event envelope.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test --workspace m07_external_trace m07_traceability m07_mcp_skill_traceability
```

Expected: fails because trace payloads do not exist.

### Task 3: Add Runtime Trace Payloads

**Files:**
- Create: `crates/ruse-types/src/external.rs`
- Modify: `crates/ruse-types/src/event.rs`
- Modify: `crates/ruse-types/src/lib.rs`

- [x] **Step 1: Add external trace contracts**

Implement the payload structs from the spec in `external.rs`.

- [x] **Step 2: Extend `RuntimeEventPayload`**

Add variants:

```rust
McpServerHealthRecorded(crate::external::McpServerHealthRecorded),
McpToolsDiscovered(crate::external::McpToolsDiscovered),
McpPolicyMappingRecorded(crate::external::McpPolicyMappingRecorded),
SkillDiscoveredRecorded(crate::external::SkillDiscoveredRecorded),
SkillPromptContributionRecorded(crate::external::SkillPromptContributionRecorded),
```

- [x] **Step 3: Export `external` module**

Update `ruse-types/src/lib.rs`.

- [x] **Step 4: Run ruse-types tests**

Run:

```bash
cargo test -p ruse-types m07_external_trace
```

Expected: trace serialization tests pass.

### Task 4: Normalize Trace Payloads for UI

**Files:**
- Modify: `crates/ruse-ui-bridge/src/event.rs`
- Modify: `crates/ruse-ui-bridge/tests/m07_traceability.rs`

- [x] **Step 1: Add UI payload variants or progress mapping**

Represent trace payloads with safe summaries that preserve source IDs, counts, local tool names, permission action, and source paths.

- [x] **Step 2: Ensure raw prompt body and schemas are absent**

Assertions must serialize the UI payload and check that known raw prompt body and schema strings are absent.

- [x] **Step 3: Verify UI tests**

Run:

```bash
cargo test -p ruse-ui-bridge m07_traceability
```

Expected: UI traceability tests pass.

### Task 5: Verify M07 Integration and Completion

**Files:**
- Modify: `docs/plans/M07-04-mcp-skill-traceability.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Run focused core test**

Run:

```bash
cargo test -p ruse-core m07_mcp_skill_traceability
```

Expected: core trace fixture test passes.

- [x] **Step 2: Mark M07 complete**

Update milestone status so M07 slice specs, plans, and implementation are complete. Advance active focus to M08 planning.

- [x] **Step 3: Run standard verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 4: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m07 mcp skills"
```

Expected: commit succeeds with source, tests, and documentation together.
