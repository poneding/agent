# M11 Plugin Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Open the runtime to controlled third-party extension through manifests, process isolation, declared capabilities, tools, hooks, and a WASM prototype.

**Architecture:** Implement this milestone behind the crate boundaries named in the matching spec. Keep shared contracts in lower-level crates, route orchestration through `agent-core`, and expose user-facing behavior through CLI or UI bridge crates only when the spec calls for it.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde-compatible contracts where durable or wire formats are involved, Tokio for async runtime paths that need it, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish Technical Design And Slice Plans

**Files:**
- Read: `docs/specs/M11-plugin-runtime.spec.md`
- Create: `docs/specs/M11-00-plugin-runtime-technical-design.spec.md`
- Create: `docs/plans/M11-00-plugin-runtime-technical-design.plan.md`
- Create: `docs/specs/M11-01-plugin-manifest.spec.md`
- Create: `docs/plans/M11-01-plugin-manifest.plan.md`
- Create: `docs/specs/M11-02-process-plugin-host.spec.md`
- Create: `docs/plans/M11-02-process-plugin-host.plan.md`
- Create: `docs/specs/M11-03-plugin-capabilities.spec.md`
- Create: `docs/plans/M11-03-plugin-capabilities.plan.md`
- Create: `docs/specs/M11-04-plugin-tools-hooks.spec.md`
- Create: `docs/plans/M11-04-plugin-tools-hooks.plan.md`
- Create: `docs/specs/M11-05-wasm-prototype.spec.md`
- Create: `docs/plans/M11-05-wasm-prototype.plan.md`

- [x] **Step 1: Write M11 technical design**

The design defines manifest schema, version gates, isolation modes, process protocol, capability mapping, plugin tool registration, hook rules, WASM prototype constraints, security, and tests.

- [x] **Step 2: Write M11 slice specs and plans**

Each slice has a milestone-prefixed spec and plan with concrete files, contracts, test names, verification commands, and commit boundaries.

### Task 2: Implement M11-01 Plugin Manifest

**Files:**
- Read: `docs/specs/M11-01-plugin-manifest.spec.md`
- Execute: `docs/plans/M11-01-plugin-manifest.plan.md`

- [ ] **Step 1: Follow M11-01 plan**

Run:

```bash
sed -n '1,260p' docs/plans/M11-01-plugin-manifest.plan.md
```

Expected: the plan describes the RED tests, implementation files, verification commands, and commit for plugin manifest support.

### Task 3: Implement M11-02 Process Plugin Host

**Files:**
- Read: `docs/specs/M11-02-process-plugin-host.spec.md`
- Execute: `docs/plans/M11-02-process-plugin-host.plan.md`

- [ ] **Step 1: Follow M11-02 plan**

Run:

```bash
sed -n '1,260p' docs/plans/M11-02-process-plugin-host.plan.md
```

Expected: the plan describes process protocol tests, host implementation, verification commands, and commit.

### Task 4: Implement M11-03 Plugin Capabilities

**Files:**
- Read: `docs/specs/M11-03-plugin-capabilities.spec.md`
- Execute: `docs/plans/M11-03-plugin-capabilities.plan.md`

- [ ] **Step 1: Follow M11-03 plan**

Run:

```bash
sed -n '1,260p' docs/plans/M11-03-plugin-capabilities.plan.md
```

Expected: the plan describes capability validation, policy request mapping, event tests, verification commands, and commit.

### Task 5: Implement M11-04 Plugin Tools And Hooks

**Files:**
- Read: `docs/specs/M11-04-plugin-tools-hooks.spec.md`
- Execute: `docs/plans/M11-04-plugin-tools-hooks.plan.md`

- [ ] **Step 1: Follow M11-04 plan**

Run:

```bash
sed -n '1,260p' docs/plans/M11-04-plugin-tools-hooks.plan.md
```

Expected: the plan describes plugin tool registration, hook contracts, core integration tests, verification commands, and commit.

### Task 6: Implement M11-05 WASM Prototype

**Files:**
- Read: `docs/specs/M11-05-wasm-prototype.spec.md`
- Execute: `docs/plans/M11-05-wasm-prototype.plan.md`

- [ ] **Step 1: Follow M11-05 plan**

Run:

```bash
sed -n '1,260p' docs/plans/M11-05-wasm-prototype.plan.md
```

Expected: the plan describes WASM validation tests, prototype implementation, verification commands, M11 completion status update, and commit.
