# M22-02 Plugin Install Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI plugin management commands backed by `agent-plugin` validation and `agent-core` extension registry state.

**Architecture:** `agent-cli` parses and renders commands. `agent-plugin` validates manifests and compatibility. `agent-core` owns registry mutation. Install commands do not execute plugin code.

**Tech Stack:** Rust 2024 edition, serde_json fixture manifests, assert command tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M22-02-plugin-install-commands.spec.md`
- Read: `docs/specs/M22-01-extension-registry.spec.md`
- Read: `crates/agent-plugin/src/manifest.rs`
- Read: `crates/agent-plugin/src/capability.rs`

- [ ] **Step 1: Review plugin UX scope**

Run:

```bash
sed -n '1,260p' docs/specs/M22-02-plugin-install-commands.spec.md
```

Expected: commands, capability review, failure handling, and fixture requirements are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-cli/tests/plugin_commands.rs`
- Create: `crates/agent-plugin/tests/fixtures/hello-process-plugin/plugin.json`
- Create: `crates/agent-plugin/tests/fixtures/hello-process-plugin/plugin.sh`
- Create: `crates/agent-plugin/tests/fixtures/invalid-plugin/plugin.json`

- [ ] **Step 1: Add parser and output tests**

Cover `install`, `list`, `inspect`, `enable`, `disable`, and `remove` in text and JSON modes.

- [ ] **Step 2: Add capability review tests**

Cover enable failure without `--accept-capabilities` and success with explicit acceptance.

- [ ] **Step 3: Add invalid fixture tests**

Cover missing path, invalid manifest, unsupported compatibility, and duplicate plugin id.

- [ ] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-cli plugin_commands
```

Expected: tests fail because commands are missing.

### Task 3: Implement Plugin Commands

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-plugin/src/lib.rs`
- Modify: `crates/agent-plugin/src/manifest.rs`

- [ ] **Step 1: Add command parsing**

Add plugin subcommands without changing existing prompt, JSON, and TUI entrypoint behavior.

- [ ] **Step 2: Add install report builder**

Build `PluginInstallRequest` and `PluginInstallReport` from plugin manifest validation and registry mutation.

- [ ] **Step 3: Add enable/disable/remove flows**

Use M22-01 registry transitions and enforce explicit capability review before enabling.

- [ ] **Step 4: Add safe renderers**

Render stable text and JSON outputs without raw env values or secret-like diagnostics.

### Task 4: Verify

**Files:**
- Read: `crates/agent-cli/tests/plugin_commands.rs`

- [ ] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-cli plugin_commands
```

Expected: focused plugin command tests pass.

- [ ] **Step 2: Run workspace checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `docs/plans/M22-02-plugin-install-commands.plan.md`
- Modify: `docs/plans/M22-extension-ux-mcp-gateway.plan.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Mark M22-02 complete**

Advance active implementation focus to M22-03.

- [ ] **Step 2: Commit M22-02**

Run:

```bash
git add crates/agent-cli crates/agent-plugin docs Cargo.lock
git commit -m "feat: add plugin install commands"
```

Expected: commit succeeds.
