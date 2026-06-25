# M11-03 Plugin Capabilities Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add plugin capability validation and policy request mapping.

**Architecture:** Keep validation in `agent-plugin`; keep decisions in `agent-policy`; add only durable trace payloads to `agent-types` when needed.

**Tech Stack:** Rust 2024 edition, serde, existing policy/tool/types crates, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M11-03-plugin-capabilities.spec.md`
- Read: `crates/agent-policy/src/capability.rs`
- Read: `crates/agent-policy/src/request.rs`

- [x] **Step 1: Review capability mapping**

Run:

```bash
sed -n '1,260p' docs/specs/M11-03-plugin-capabilities.spec.md
```

Expected: the spec maps plugin tool use into the current policy model.

### Task 2: Write RED Capability Tests

**Files:**
- Create: `crates/agent-plugin/tests/capabilities.rs`
- Modify/Create: `crates/agent-types/tests/m11_plugin_capabilities.rs`

- [x] **Step 1: Add failing tests**

Add tests:

- `plugin_capabilities_validate_tool_and_hook_declarations`
- `plugin_capabilities_reject_missing_tool_capability`
- `plugin_capabilities_reject_missing_hook_capability`
- `plugin_capabilities_reject_duplicate_capability`
- `plugin_capabilities_reject_empty_permission_parts`
- `plugin_capabilities_tool_permission_request_maps_to_tool_scope`
- `m11_plugin_capabilities_event_serializes_stable_shape`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-plugin capabilities
cargo test -p agent-types m11_plugin_capabilities
```

Expected: tests fail because capability validation and event payloads do not exist yet.

### Task 3: Implement Capability Contracts

**Files:**
- Modify: `crates/agent-plugin/src/lib.rs`
- Create: `crates/agent-plugin/src/capability.rs`
- Modify: `crates/agent-plugin/src/manifest.rs`
- Modify: `crates/agent-plugin/src/error.rs`
- Modify: `crates/agent-types/src/external.rs`
- Modify: `crates/agent-types/src/event.rs`

- [x] **Step 1: Add capability types**

Implement capability kind, permission, and capability set contracts from the spec.

- [x] **Step 2: Add validation**

Implement validation for matching tool/hook declarations, duplicate capabilities, and non-empty permission parts.

- [x] **Step 3: Add policy request mapping**

Implement `permission_request_for_plugin_tool` using:

- `CapabilityAction::Read`
- `ResourceKind::Tool`
- resource pattern equal to the local tool name
- reason string `plugin tool execution requested: <plugin_name>`

- [x] **Step 4: Add event payload**

Add a stable external event payload for plugin capability validation summary.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/agent-plugin/tests/capabilities.rs`
- Read: `crates/agent-types/tests/m11_plugin_capabilities.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-plugin capabilities
cargo test -p agent-types m11_plugin_capabilities
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M11-03-plugin-capabilities.plan.md`

- [x] **Step 1: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m11 plugin capabilities"
```

Expected: commit succeeds.
