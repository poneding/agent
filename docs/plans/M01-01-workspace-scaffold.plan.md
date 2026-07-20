# M01-01 Workspace Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the initial Rust workspace and runtime-spine crate skeleton for Milestone 1.

**Architecture:** The workspace starts with five crates: `ruse-types`, `ruse-model`, `ruse-storage`, `ruse-core`, and `ruse-cli`. Dependencies flow inward to `ruse-types`, while `ruse-cli` depends on `ruse-core`; the scaffold proves crate linkage without defining the event model yet.

**Tech Stack:** Rust 2024 edition, Cargo workspace resolver 2, standard library only, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Create Root Workspace Manifest

**Files:**
- Create: `Cargo.toml`

- [ ] **Step 1: Run metadata before the workspace exists**

Run:

```bash
cargo metadata --no-deps
```

Expected: command fails because `Cargo.toml` does not exist.

- [ ] **Step 2: Create the root workspace manifest**

Create `Cargo.toml` with:

```toml
[workspace]
members = []
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "MIT"
authors = ["Pone Ding"]

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "warn"
```

- [ ] **Step 3: Run metadata and verify the empty workspace**

Run:

```bash
cargo metadata --no-deps
```

Expected: command succeeds and reports an empty workspace package list.

- [ ] **Step 4: Commit the workspace manifest**

```bash
git add Cargo.toml
git commit -m "chore: define runtime workspace"
```

### Task 2: Create `ruse-types`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-types/Cargo.toml`
- Create: `crates/ruse-types/src/lib.rs`

- [ ] **Step 1: Add `ruse-types` to the workspace**

Update the root `Cargo.toml` workspace members to:

```toml
members = [
    "crates/ruse-types",
]
```

- [ ] **Step 2: Create the crate manifest**

Create `crates/ruse-types/Cargo.toml` with:

```toml
[package]
name = "ruse-types"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[lints]
workspace = true
```

- [ ] **Step 3: Write the failing crate identity test**

Create `crates/ruse-types/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Shared types for the agent.

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_identity() {
        assert_eq!(crate::crate_name(), "ruse-types");
    }
}
```

- [ ] **Step 4: Run the test and verify failure**

Run:

```bash
cargo test -p ruse-types exposes_crate_identity
```

Expected: compilation fails with a missing `crate_name` function.

- [ ] **Step 5: Implement the minimal API**

Replace `crates/ruse-types/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Shared types for the agent.

/// Returns this crate's stable package name.
#[must_use]
pub const fn crate_name() -> &'static str {
    "ruse-types"
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_identity() {
        assert_eq!(crate::crate_name(), "ruse-types");
    }
}
```

- [ ] **Step 6: Run the test and verify success**

Run:

```bash
cargo test -p ruse-types exposes_crate_identity
```

Expected: test passes.

- [ ] **Step 7: Commit `ruse-types`**

```bash
git add Cargo.toml crates/ruse-types
git commit -m "feat: add ruse-types crate"
```

### Task 3: Create `ruse-model`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-model/Cargo.toml`
- Create: `crates/ruse-model/src/lib.rs`

- [ ] **Step 1: Add `ruse-model` to the workspace**

Update the root `Cargo.toml` workspace members to:

```toml
members = [
    "crates/ruse-types",
    "crates/ruse-model",
]
```

- [ ] **Step 2: Create the crate manifest**

Create `crates/ruse-model/Cargo.toml` with:

```toml
[package]
name = "ruse-model"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
ruse-types = { path = "../ruse-types" }

[lints]
workspace = true
```

- [ ] **Step 3: Write the failing model layer test**

Create `crates/ruse-model/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Model provider abstractions for the agent.

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_model_layer_identity() {
        assert_eq!(crate::model_layer_name(), "ruse-model uses ruse-types");
    }
}
```

- [ ] **Step 4: Run the test and verify failure**

Run:

```bash
cargo test -p ruse-model exposes_model_layer_identity
```

Expected: compilation fails with a missing `model_layer_name` function.

- [ ] **Step 5: Implement the minimal API**

Replace `crates/ruse-model/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Model provider abstractions for the agent.

/// Returns a smoke-test string proving the model crate links shared types.
#[must_use]
pub fn model_layer_name() -> String {
    format!("ruse-model uses {}", ruse_types::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_model_layer_identity() {
        assert_eq!(crate::model_layer_name(), "ruse-model uses ruse-types");
    }
}
```

- [ ] **Step 6: Run the test and verify success**

Run:

```bash
cargo test -p ruse-model exposes_model_layer_identity
```

Expected: test passes.

- [ ] **Step 7: Commit `ruse-model`**

```bash
git add Cargo.toml crates/ruse-model
git commit -m "feat: add ruse-model crate"
```

### Task 4: Create `ruse-storage`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-storage/Cargo.toml`
- Create: `crates/ruse-storage/src/lib.rs`

- [ ] **Step 1: Add `ruse-storage` to the workspace**

Update the root `Cargo.toml` workspace members to:

```toml
members = [
    "crates/ruse-types",
    "crates/ruse-model",
    "crates/ruse-storage",
]
```

- [ ] **Step 2: Create the crate manifest**

Create `crates/ruse-storage/Cargo.toml` with:

```toml
[package]
name = "ruse-storage"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
ruse-types = { path = "../ruse-types" }

[lints]
workspace = true
```

- [ ] **Step 3: Write the failing storage layer test**

Create `crates/ruse-storage/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Storage abstractions for runtime events, checkpoints, and artifacts.

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_storage_layer_identity() {
        assert_eq!(crate::storage_layer_name(), "ruse-storage uses ruse-types");
    }
}
```

- [ ] **Step 4: Run the test and verify failure**

Run:

```bash
cargo test -p ruse-storage exposes_storage_layer_identity
```

Expected: compilation fails with a missing `storage_layer_name` function.

- [ ] **Step 5: Implement the minimal API**

Replace `crates/ruse-storage/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Storage abstractions for runtime events, checkpoints, and artifacts.

/// Returns a smoke-test string proving the storage crate links shared types.
#[must_use]
pub fn storage_layer_name() -> String {
    format!("ruse-storage uses {}", ruse_types::crate_name())
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_storage_layer_identity() {
        assert_eq!(crate::storage_layer_name(), "ruse-storage uses ruse-types");
    }
}
```

- [ ] **Step 6: Run the test and verify success**

Run:

```bash
cargo test -p ruse-storage exposes_storage_layer_identity
```

Expected: test passes.

- [ ] **Step 7: Commit `ruse-storage`**

```bash
git add Cargo.toml crates/ruse-storage
git commit -m "feat: add ruse-storage crate"
```

### Task 5: Create `ruse-core`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-core/Cargo.toml`
- Create: `crates/ruse-core/src/lib.rs`

- [ ] **Step 1: Add `ruse-core` to the workspace**

Update the root `Cargo.toml` workspace members to:

```toml
members = [
    "crates/ruse-types",
    "crates/ruse-model",
    "crates/ruse-storage",
    "crates/ruse-core",
]
```

- [ ] **Step 2: Create the crate manifest**

Create `crates/ruse-core/Cargo.toml` with:

```toml
[package]
name = "ruse-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
ruse-model = { path = "../ruse-model" }
ruse-storage = { path = "../ruse-storage" }
ruse-types = { path = "../ruse-types" }

[lints]
workspace = true
```

- [ ] **Step 3: Write the failing core linkage test**

Create `crates/ruse-core/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Core runtime orchestration for the agent.

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_runtime_banner() {
        assert_eq!(
            crate::runtime_banner(),
            "agent (ruse-model uses ruse-types, ruse-storage uses ruse-types)"
        );
    }
}
```

- [ ] **Step 4: Run the test and verify failure**

Run:

```bash
cargo test -p ruse-core exposes_runtime_banner
```

Expected: compilation fails with a missing `runtime_banner` function.

- [ ] **Step 5: Implement the minimal API**

Replace `crates/ruse-core/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]
//! Core runtime orchestration for the agent.

/// Returns a smoke-test banner proving the core crate links model and storage.
#[must_use]
pub fn runtime_banner() -> String {
    format!(
        "agent ({}, {})",
        ruse_model::model_layer_name(),
        ruse_storage::storage_layer_name()
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_runtime_banner() {
        assert_eq!(
            crate::runtime_banner(),
            "agent (ruse-model uses ruse-types, ruse-storage uses ruse-types)"
        );
    }
}
```

- [ ] **Step 6: Run the test and verify success**

Run:

```bash
cargo test -p ruse-core exposes_runtime_banner
```

Expected: test passes.

- [ ] **Step 7: Commit `ruse-core`**

```bash
git add Cargo.toml crates/ruse-core
git commit -m "feat: add ruse-core crate"
```

### Task 6: Create `ruse-cli`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-cli/Cargo.toml`
- Create: `crates/ruse-cli/src/main.rs`
- Create: `crates/ruse-cli/tests/cli_smoke.rs`

- [ ] **Step 1: Add `ruse-cli` to the workspace**

Update the root `Cargo.toml` workspace members to:

```toml
members = [
    "crates/ruse-types",
    "crates/ruse-model",
    "crates/ruse-storage",
    "crates/ruse-core",
    "crates/ruse-cli",
]
```

- [ ] **Step 2: Create the crate manifest**

Create `crates/ruse-cli/Cargo.toml` with:

```toml
[package]
name = "ruse-cli"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
ruse-core = { path = "../ruse-core" }

[lints]
workspace = true
```

- [ ] **Step 3: Write the failing CLI smoke test**

Create `crates/ruse-cli/tests/cli_smoke.rs` with:

```rust
use std::process::Command;

#[test]
fn prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .arg("--version")
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ruse-cli 0.1.0\n");
    assert!(output.stderr.is_empty());
}
```

Create `crates/ruse-cli/src/main.rs` with:

```rust
#![forbid(unsafe_code)]

fn main() {}
```

- [ ] **Step 4: Run the test and verify failure**

Run:

```bash
cargo test -p ruse-cli prints_version
```

Expected: test fails because the binary does not print the version string.

- [ ] **Step 5: Implement the CLI smoke behavior**

Replace `crates/ruse-cli/src/main.rs` with:

```rust
#![forbid(unsafe_code)]

fn main() {
    let mut args = std::env::args().skip(1);

    if matches!(args.next().as_deref(), Some("--version" | "-V")) {
        println!("ruse-cli {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    println!("{}", ruse_core::runtime_banner());
}
```

- [ ] **Step 6: Run the test and verify success**

Run:

```bash
cargo test -p ruse-cli prints_version
```

Expected: test passes.

- [ ] **Step 7: Verify default CLI output**

Run:

```bash
cargo run -p ruse-cli
```

Expected stdout:

```text
agent (ruse-model uses ruse-types, ruse-storage uses ruse-types)
```

- [ ] **Step 8: Commit `ruse-cli`**

```bash
git add Cargo.toml crates/ruse-cli
git commit -m "feat: add ruse-cli smoke binary"
```

### Task 7: Run Workspace Verification

**Files:**
- Modify: `Cargo.lock`

- [ ] **Step 1: Format the workspace**

Run:

```bash
cargo fmt --all
```

Expected: command succeeds.

- [ ] **Step 2: Verify formatting**

Run:

```bash
cargo fmt --all -- --check
```

Expected: command succeeds.

- [ ] **Step 3: Run clippy**

Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: command succeeds without warnings.

- [ ] **Step 4: Run tests**

Run:

```bash
cargo test --workspace
```

Expected: all crate unit tests and CLI integration tests pass.

- [ ] **Step 5: Verify metadata**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: output includes the five packages `ruse-types`, `ruse-model`, `ruse-storage`, `ruse-core`, and `ruse-cli`.

- [ ] **Step 6: Commit generated lockfile and formatted scaffold**

```bash
git add Cargo.lock Cargo.toml crates
git commit -m "test: verify workspace scaffold"
```

### Task 8: Update Documentation Links

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`

- [ ] **Step 1: Update README documentation links**

Ensure the `README.md` Documentation section contains:

```markdown
## Documentation

- [Product Requirements Document](docs/PRD.md)
- [Milestones](docs/milestones/README.md)
- [M01-01 Workspace Scaffold Spec](docs/specs/M01-01-workspace-scaffold.spec.md)
- [M01-01 Workspace Scaffold Plan](docs/plans/M01-01-workspace-scaffold.plan.md)
- [Agent Working Guide](AGENTS.md)
```

- [ ] **Step 2: Update AGENTS documentation guidance**

Ensure `AGENTS.md` includes these bullets under `Documentation Guidance`:

```markdown
- Update `docs/milestones/README.md` when milestone order, slice boundaries, or active focus changes.
- Add or update `docs/specs/Mxx-yy-topic.spec.md` before implementing a milestone slice.
- Add or update `docs/plans/Mxx-yy-topic.plan.md` after the matching spec is accepted.
```

- [ ] **Step 3: Commit documentation updates**

```bash
git add README.md AGENTS.md docs/milestones/README.md docs/specs/M01-01-workspace-scaffold.spec.md docs/plans/M01-01-workspace-scaffold.plan.md
git commit -m "docs: add M01 workspace scaffold spec and plan"
```

### Task 9: Final M01-01 Acceptance Check

**Files:**
- Read: `docs/specs/M01-01-workspace-scaffold.spec.md`
- Read: `docs/plans/M01-01-workspace-scaffold.plan.md`

- [ ] **Step 1: Run the full verification command set**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p ruse-cli -- --version
```

Expected final command output:

```text
ruse-cli 0.1.0
```

- [ ] **Step 2: Confirm M01-01 acceptance criteria**

Confirm:

```text
cargo metadata --no-deps succeeds.
cargo fmt --all -- --check succeeds.
cargo clippy --workspace --all-targets -- -D warnings succeeds.
cargo test --workspace succeeds.
cargo run -p ruse-cli -- --version prints ruse-cli 0.1.0.
The workspace contains only runtime-spine crates needed by M01-01.
```

- [ ] **Step 3: Record the next active slice**

Update `docs/milestones/README.md` active work from:

```markdown
Current focus: `M01-01-workspace-scaffold`.
```

to:

```markdown
Current focus: `M01-02-event-model`.
```

- [ ] **Step 4: Commit the active focus update**

```bash
git add docs/milestones/README.md
git commit -m "docs: advance active milestone focus"
```
