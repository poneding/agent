# M11-01 Plugin Manifest Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `agent-plugin` and implement typed plugin manifest parsing, validation, and compatibility checks.

**Architecture:** Keep plugin-specific contracts in `agent-plugin`. The crate may depend on `agent-tool`, `agent-policy`, `agent-types`, `serde`, `serde_json`, and `thiserror`; it must not depend on `agent-core` or UI crates.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `serde_json`, `thiserror`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M11-00-plugin-runtime-technical-design.spec.md`
- Read: `docs/specs/M11-01-plugin-manifest.spec.md`
- Read: `docs/templates/plugin-manifest.template.json`
- Read: `docs/architecture/plugin-system.md`

- [x] **Step 1: Review the technical design and slice spec**

Run:

```bash
sed -n '1,260p' docs/specs/M11-00-plugin-runtime-technical-design.spec.md
sed -n '1,260p' docs/specs/M11-01-plugin-manifest.spec.md
```

Expected: both documents define manifest ownership, schema, compatibility, validation, and test requirements.

### Task 2: Write RED Manifest Tests

**Files:**
- Create: `crates/agent-plugin/tests/manifest.rs`

- [x] **Step 1: Create the manifest tests before implementation**

Add tests with these names:

```rust
#[test]
fn manifest_parses_template_and_validates() {}

#[test]
fn manifest_rejects_unsupported_schema_version() {}

#[test]
fn manifest_rejects_unsupported_api_version() {}

#[test]
fn manifest_rejects_invalid_plugin_name() {}

#[test]
fn manifest_rejects_missing_process_entrypoint() {}

#[test]
fn manifest_rejects_builtin_entrypoint() {}

#[test]
fn manifest_rejects_duplicate_tools_and_hooks() {}

#[test]
fn manifest_serializes_stable_shape() {}
```

Use `include_str!("../../../docs/templates/plugin-manifest.template.json")` for the valid template fixture. Use `serde_json::Value` mutation helpers inside the test file to modify invalid cases.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-plugin manifest
```

Expected: command fails because `agent-plugin` is not yet a workspace crate.

### Task 3: Add `agent-plugin` Crate Skeleton

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/agent-plugin/Cargo.toml`
- Create: `crates/agent-plugin/src/lib.rs`
- Create: `crates/agent-plugin/src/error.rs`
- Create: `crates/agent-plugin/src/compatibility.rs`
- Create: `crates/agent-plugin/src/manifest.rs`

- [x] **Step 1: Add workspace member**

Add `"crates/agent-plugin"` to the workspace members after `agent-skill` and before `agent-ui-bridge`.

- [x] **Step 2: Add crate manifest**

Use this dependency shape:

```toml
[dependencies]
agent-policy = { path = "../agent-policy" }
agent-tool = { path = "../agent-tool" }
agent-types = { path = "../agent-types" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
```

- [x] **Step 3: Export modules**

`src/lib.rs` exports:

```rust
#![forbid(unsafe_code)]
//! Plugin runtime contracts for agent.

pub mod compatibility;
pub mod error;
pub mod manifest;

pub use compatibility::{
    PLUGIN_MANIFEST_SCHEMA_VERSION, PLUGIN_RUNTIME_API_VERSION, check_plugin_compatibility,
};
pub use error::PluginManifestError;
pub use manifest::{
    PluginCapabilityManifest, PluginHookManifest, PluginIsolation, PluginIsolationMode,
    PluginManifest, PluginName, PluginRuntimeCompatibility, PluginToolManifest,
    parse_plugin_manifest_json, validate_plugin_manifest,
};
```

### Task 4: Implement Manifest Contracts

**Files:**
- Modify: `crates/agent-plugin/src/error.rs`
- Modify: `crates/agent-plugin/src/compatibility.rs`
- Modify: `crates/agent-plugin/src/manifest.rs`

- [x] **Step 1: Add typed errors**

Implement `PluginManifestError` with variants:

- `Parse(String)`
- `InvalidName(String)`
- `UnsupportedSchema { found: u32, supported: u32 }`
- `IncompatibleRuntime(String)`
- `InvalidIsolation(String)`
- `DuplicateContribution(String)`
- `InvalidManifest(String)`

- [x] **Step 2: Add compatibility checks**

Implement constants:

```rust
pub const PLUGIN_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const PLUGIN_RUNTIME_API_VERSION: u32 = 1;
```

Implement `check_plugin_compatibility(&PluginRuntimeCompatibility) -> Result<(), PluginManifestError>` so API versions other than `1` fail.

- [x] **Step 3: Add manifest types and validation**

Implement serde-compatible structs and validation from the spec:

- plugin name cannot be empty
- plugin name uses lowercase ASCII, digits, `_`, `-`, `.`
- process and WASM require non-empty entrypoint
- built-in rejects entrypoint
- tool names are validated through `agent_tool::ToolName`
- duplicate tool names fail
- duplicate hook names fail
- `configuration_schema` must be a JSON object

### Task 5: Verify GREEN

**Files:**
- Read: `crates/agent-plugin/tests/manifest.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-plugin manifest
```

Expected: all manifest tests pass.

- [x] **Step 2: Run workspace gates**

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
- Modify: `docs/plans/M11-01-plugin-manifest.plan.md`

- [x] **Step 1: Mark plan items complete**

Check off completed steps in this plan.

- [x] **Step 2: Update milestone status**

Mark `M11-01 implementation` complete and set active implementation focus to `M11-02-process-plugin-host`.

- [ ] **Step 3: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m11 plugin manifest"
```

Expected: commit succeeds with source, tests, and docs staged together.
