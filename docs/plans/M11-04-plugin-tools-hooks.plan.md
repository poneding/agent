# M11-04 Plugin Tools And Hooks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Register plugin tools into `ToolRegistry`, invoke them through a host adapter, and add bounded hook contracts.

**Architecture:** Plugin tool adapters live in `agent-plugin`; runtime policy/audit remains in `agent-core::ToolLoop`; integration tests prove the shared tool path.

**Tech Stack:** Rust 2024 edition, `Arc`, existing tool/policy/core crates, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M11-04-plugin-tools-hooks.spec.md`
- Read: `crates/agent-tool/src/tool.rs`
- Read: `crates/agent-tool/src/registry.rs`
- Read: `crates/agent-core/src/tool_loop.rs`

- [ ] **Step 1: Review registry and tool loop contracts**

Run:

```bash
sed -n '1,260p' docs/specs/M11-04-plugin-tools-hooks.spec.md
sed -n '1,220p' crates/agent-core/src/tool_loop.rs
```

Expected: plugin tools can use the existing `Tool` trait and `ToolLoop`.

### Task 2: Write RED Plugin Tool And Hook Tests

**Files:**
- Create: `crates/agent-plugin/tests/plugin_tools_hooks.rs`
- Create: `crates/agent-core/tests/m11_plugin_runtime_integration.rs`

- [ ] **Step 1: Add failing plugin crate tests**

Add tests:

- `plugin_tool_definition_matches_manifest`
- `plugin_tool_registers_in_tool_registry`
- `plugin_tool_rejects_duplicate_registration`
- `plugin_tool_invokes_host_and_returns_output`
- `plugin_tool_maps_host_error_to_tool_error`
- `plugin_hook_runner_records_success_and_failure`

- [ ] **Step 2: Add failing core integration test**

Add test:

- `m11_plugin_tool_runs_through_policy_and_runtime_events`

The integration test should register a plugin tool, configure `StaticPolicyEngine` to allow the tool scope, run `ToolLoop`, and assert event payload order includes proposal, permission decision, execution start, execution completion, and observation.

- [ ] **Step 3: Verify RED**

Run:

```bash
cargo test -p agent-plugin plugin_tools_hooks
cargo test -p agent-core m11_plugin_runtime_integration
```

Expected: tests fail because plugin tool and hook contracts do not exist yet.

### Task 3: Implement Plugin Tools

**Files:**
- Modify: `crates/agent-plugin/src/lib.rs`
- Create: `crates/agent-plugin/src/tool.rs`
- Modify: `crates/agent-plugin/src/error.rs`

- [ ] **Step 1: Add `PluginToolHost` and invocation types**

Define a host trait that returns `ToolOutput` and receives plugin name, tool name, call id, and input JSON.

- [ ] **Step 2: Implement `PluginTool`**

Implement `agent_tool::Tool` for `PluginTool`.

- [ ] **Step 3: Implement registration helper**

Implement `register_plugin_tools` and return registered names.

### Task 4: Implement Hooks

**Files:**
- Create: `crates/agent-plugin/src/hook.rs`

- [ ] **Step 1: Add hook event and outcome types**

Implement serde-compatible hook event names and result summaries.

- [ ] **Step 2: Add hook runner**

Implement a deterministic in-memory runner that accepts hook callbacks for M11 tests and records success/failure outcomes.

### Task 5: Verify GREEN

**Files:**
- Read: `crates/agent-plugin/tests/plugin_tools_hooks.rs`
- Read: `crates/agent-core/tests/m11_plugin_runtime_integration.rs`

- [ ] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-plugin plugin_tools_hooks
cargo test -p agent-core m11_plugin_runtime_integration
```

Expected: focused tests pass.

- [ ] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 6: Update Status And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M11-04-plugin-tools-hooks.plan.md`

- [ ] **Step 1: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m11 plugin tools and hooks"
```

Expected: commit succeeds.
