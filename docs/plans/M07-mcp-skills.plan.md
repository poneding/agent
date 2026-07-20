# M07 MCP and Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add standard external capability discovery through MCP servers and skill roots.

**Architecture:** Implement this milestone behind the crate boundaries named in the matching spec. Keep shared contracts in lower-level crates, route orchestration through `ruse-core`, and expose user-facing behavior through CLI or UI bridge crates only when the spec calls for it.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde-compatible contracts where durable or wire formats are involved, Tokio for async runtime paths that need it, Cargo fmt, Cargo clippy, Cargo test.

---

## Slice Execution Order

Do not implement this milestone directly from the milestone-level plan. Execute the slice plans in order:

1. `docs/plans/M07-01-mcp-client.plan.md`
2. `docs/plans/M07-02-mcp-policy-mapping.plan.md`
3. `docs/plans/M07-03-skill-discovery.plan.md`
4. `docs/plans/M07-04-mcp-skill-traceability.plan.md`

Each slice must be implemented with RED/GREEN tests, standard verification, progress updates, and a dedicated commit.

### Task 1: Confirm Milestone Inputs

**Files:**
- Read: `docs/specs/M07-mcp-skills.spec.md`
- Read: `docs/milestones/README.md`
- Read: `docs/PRD.md`

- [ ] **Step 1: Review the milestone spec**

Run:

```bash
sed -n '1,260p' docs/specs/M07-mcp-skills.spec.md
```

Expected: the spec describes purpose, scope, contracts, tests, and acceptance criteria for M07.

- [ ] **Step 2: Confirm workspace health before changes**

Run:

```bash
cargo test --workspace
```

Expected: existing workspace tests pass before this milestone begins.

### Task 2: Add Contract Tests First

**Files:**
- Create/Modify: `crates/ruse-core/tests/m07_mcp_skills.rs`

- [ ] **Step 1: Write tests for milestone behavior**

Cover these behaviors:

- MCP discovery fixture tests
- MCP policy mapping tests
- skill manifest parser tests
- skill prompt source tracking tests

Create focused tests with names that describe the contract being proven.

Recommended test names:

- `m07_mcp_skills_validates_public_contracts`
- `m07_mcp_skills_rejects_invalid_inputs`
- `m07_mcp_skills_emits_expected_runtime_events`
- `m07_mcp_skills_preserves_serialized_shape`
- `m07_mcp_skills_reports_recoverable_errors`

- [ ] **Step 2: Run the focused tests and verify failure**

Run:

```bash
cargo test --workspace m07_mcp_skills
```

Expected: tests fail because the milestone contracts are not implemented yet.

### Task 3: Implement Milestone Contracts

**Files:**
- Modify/Create: `crates/ruse-mcp/src/lib.rs`
- Modify/Create: `crates/ruse-skill/src/lib.rs`
- Modify/Create: `crates/ruse-tool/src/lib.rs`
- Modify/Create: `crates/ruse-policy/src/lib.rs`
- Modify/Create: `crates/ruse-context/src/lib.rs`
- Modify/Create: `crates/ruse-types/src/lib.rs`

- [ ] **Step 1: Add or extend crate modules**

Implement the public contracts listed in the spec:

- McpServerConfig contract
- McpToolDescriptor contract
- McpPolicyMapping contract
- SkillManifest contract
- SkillPromptContribution contract

- [ ] **Step 2: Keep dependency direction clean**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: metadata succeeds and newly touched crates do not introduce reverse dependencies into lower-level crates.

- [ ] **Step 3: Run focused tests and verify success**

Run:

```bash
cargo test --workspace m07_mcp_skills
```

Expected: focused milestone tests pass.

### Task 4: Add Integration Coverage

**Files:**
- Create/Modify: `crates/ruse-core/tests/m07_mcp_skills_integration.rs`

- [ ] **Step 1: Add integration assertions for acceptance criteria**

Acceptance criteria to encode:

- Runtime discovers MCP tools and exposes them as permissioned local tools.
- Skills influence context assembly with traceable source metadata.

- [ ] **Step 2: Run integration tests**

Run:

```bash
cargo test --workspace m07_mcp_skills_integration
```

Expected: integration tests pass and fail deterministically when the milestone behavior is removed.

### Task 5: Run Full Verification

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [ ] **Step 1: Format check**

Run:

```bash
cargo fmt --all -- --check
```

Expected: formatting check passes.

- [ ] **Step 2: Lint check**

Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: clippy passes without warnings.

- [ ] **Step 3: Full test suite**

Run:

```bash
cargo test --workspace
```

Expected: all workspace tests pass.

### Task 6: Update Progress and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [ ] **Step 1: Update active milestone status**

Update `docs/milestones/README.md` so the completed milestone is marked as complete or the active focus advances to the next milestone.

- [ ] **Step 2: Commit the milestone**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m07 mcp skills"
```

Expected: commit succeeds with source, tests, and documentation staged together.
