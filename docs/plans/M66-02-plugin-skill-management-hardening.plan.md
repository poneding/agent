# M66-02 Plugin and Skill Management Hardening Plan

> **For agentic workers:** Stay inside existing CLI/runtime management flows. This slice hardens state and diagnostics; it does not execute plugin code or build a marketplace.

**Goal:** Make plugin capability acceptance and skill root registry health visible, deterministic, and safe.

**Architecture:** `agent-plugin` validates manifests and compatibility. `agent-core` owns extension records and leases. `agent-cli` renders commands and mutates the in-memory command runtime used by tests. Skill root discovery remains in `agent-skill`.

---

### Task 1: Confirm Baseline

**Files:**
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/tests/plugin_commands.rs`
- Read: `crates/agent-cli/tests/skill_root_commands.rs`
- Read: `crates/agent-skill/src/roots.rs`
- Read: `crates/agent-plugin/tests/fixtures/`

- [x] **Step 1: Inspect existing management commands**

Existing plugin commands cover install/list/inspect/enable/disable/remove. Existing skill root commands cover add/list/inspect/enable/disable/remove and registry upsert during add/inspect/enable/disable.

### Task 2: Harden Plugin Capability Acceptance

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/tests/plugin_commands.rs`

- [x] **Step 1: Add deterministic capability leases**

When `--accept-capabilities` is used during plugin install/enable, populate `ExtensionCapabilityLease` entries for declared permissions.

- [x] **Step 2: Block incompatible plugin enablement**

Reject `plugin enable` when the installed record compatibility is unsupported or unknown.

- [x] **Step 3: Add stable diagnostics**

Populate `PluginInstallReport.diagnostics` with stable review/compatibility/status codes.

### Task 3: Refresh Skill Root Registry Health

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/tests/skill_root_commands.rs`

- [x] **Step 1: Refresh registry on list**

Use current skill root inspections to refresh extension registry records when listing roots.

- [x] **Step 2: Cover duplicate skill health**

Add a deterministic test where two roots expose the same skill id and list refresh marks both extension records unhealthy.

### Task 4: Update Docs

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M66-extensions-mcp-hardening.plan.md`

- [x] **Step 1: Mark M66-02 active/complete**

Update milestone and roadmap status after implementation and checks pass.

### Task 5: Verification

**Commands:**

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-cli plugin_commands skill_root_commands
CARGO_TARGET_DIR=target/m66-dev cargo clippy -p agent-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 1: Run focused tests**

Run plugin and skill root command tests.

- [x] **Step 2: Run lint and formatting checks**

Run clippy, fmt, and whitespace checks.

### Task 6: Commit

**Files:**
- Stage only M66-02 docs, CLI code/tests, and status docs.
- Do not stage `tmp/`.

- [x] **Step 1: Commit M66-02**

Suggested message:

```text
feat: harden plugin skill management
```
