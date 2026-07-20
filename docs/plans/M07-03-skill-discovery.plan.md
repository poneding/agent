# M07-03 Skill Discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `ruse-skill` for deterministic local skill discovery and prompt contribution creation.

**Architecture:** `ruse-skill` reads local `SKILL.md` files and emits typed discovery records plus `ruse-context` prompt layers. It does not execute skill code, load arbitrary assets, or depend on runtime orchestration.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `serde_yaml`, `thiserror`, `tempfile`, existing `ruse-context`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold `ruse-skill`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-skill/Cargo.toml`
- Create: `crates/ruse-skill/src/lib.rs`
- Create: `crates/ruse-skill/src/error.rs`

- [x] **Step 1: Add crate scaffold**

Add `crates/ruse-skill` to workspace members.

Create `crates/ruse-skill/Cargo.toml`:

```toml
[package]
name = "ruse-skill"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
ruse-context = { path = "../ruse-context" }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
thiserror = "2"

[dev-dependencies]
tempfile = "3"

[lints]
workspace = true
```

Create `src/lib.rs`:

```rust
#![forbid(unsafe_code)]
//! Skill discovery and prompt contribution contracts for agent.

pub mod discovery;
pub mod error;
pub mod manifest;
pub mod prompt;

pub use discovery::{discover_skills, DiscoveredSkill, SkillDiscoveryConfig, SkillRoot, SkillSource};
pub use error::SkillError;
pub use manifest::{SkillId, SkillManifest};
pub use prompt::{skill_prompt_layer, SkillPromptContribution};
```

- [x] **Step 2: Confirm scaffold compiles**

Run:

```bash
cargo test -p ruse-skill
```

Expected: fails because modules are declared but not implemented.

### Task 2: Write Skill Discovery RED Tests

**Files:**
- Create: `crates/ruse-skill/tests/skill_discovery.rs`

- [x] **Step 1: Add failing tests**

Create tests with these names:

```rust
#[test]
fn skill_discovery_loads_valid_skill_root() {
    let temp = tempfile::tempdir().unwrap();
    let skill_dir = temp.path().join("code-review");
    std::fs::create_dir(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: Code Review\ndescription: Review code changes\nversion: 0.1.0\n---\nCheck diffs carefully.\n",
    )
    .unwrap();

    let skills = ruse_skill::discover_skills(&ruse_skill::SkillDiscoveryConfig {
        roots: vec![ruse_skill::SkillRoot {
            path: skill_dir.clone(),
            enabled: true,
        }],
    })
    .unwrap();

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].manifest.id.as_str(), "code-review");
    assert_eq!(skills[0].manifest.name, "Code Review");
    assert_eq!(skills[0].source.root_path, skill_dir);
    assert!(skills[0].prompt.content.contains("Check diffs carefully."));
}

#[test]
fn skill_discovery_ignores_disabled_root() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing");

    let skills = ruse_skill::discover_skills(&ruse_skill::SkillDiscoveryConfig {
        roots: vec![ruse_skill::SkillRoot {
            path: missing,
            enabled: false,
        }],
    })
    .unwrap();

    assert!(skills.is_empty());
}

#[test]
fn skill_discovery_rejects_invalid_manifest() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("SKILL.md"), "---\nname: Broken\n---\nBody\n").unwrap();

    let err = ruse_skill::discover_skills(&ruse_skill::SkillDiscoveryConfig {
        roots: vec![ruse_skill::SkillRoot {
            path: temp.path().to_path_buf(),
            enabled: true,
        }],
    })
    .unwrap_err();

    assert!(matches!(err, ruse_skill::SkillError::InvalidManifest(_)));
}

#[test]
fn skill_discovery_converts_prompt_to_context_layer() {
    let contribution = ruse_skill::SkillPromptContribution {
        skill_id: ruse_skill::SkillId::new("code-review").unwrap(),
        layer_id: "skill:code-review".to_string(),
        content: "Check diffs carefully.".to_string(),
        priority: 600,
        token_estimate: 4,
        source_path: std::path::PathBuf::from("/tmp/SKILL.md"),
    };

    let layer = ruse_skill::skill_prompt_layer(&contribution);

    assert_eq!(layer.id, "skill:code-review");
    assert_eq!(layer.kind, ruse_context::PromptLayerKind::Skill);
    assert_eq!(layer.priority, 600);
    assert_eq!(layer.content, "Check diffs carefully.");
}
```

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-skill skill_discovery
```

Expected: fails because skill contracts do not exist.

### Task 3: Implement Manifest and Prompt Contracts

**Files:**
- Create: `crates/ruse-skill/src/manifest.rs`
- Create: `crates/ruse-skill/src/prompt.rs`
- Modify: `crates/ruse-skill/src/error.rs`

- [x] **Step 1: Add error type**

Implement `SkillError` variants: `RootNotFound`, `ManifestNotFound`, `InvalidManifest`, `InvalidPrompt`, and `Io`.

- [x] **Step 2: Add `SkillId` and `SkillManifest`**

Implement ID validation and front matter deserialization target.

- [x] **Step 3: Add prompt contribution conversion**

Implement `SkillPromptContribution` and `skill_prompt_layer`.

- [x] **Step 4: Run focused prompt test**

Run:

```bash
cargo test -p ruse-skill skill_discovery_converts_prompt_to_context_layer
```

Expected: prompt conversion test passes.

### Task 4: Implement Skill Discovery

**Files:**
- Create: `crates/ruse-skill/src/discovery.rs`

- [x] **Step 1: Add discovery contracts**

Implement `SkillRoot`, `SkillSource`, `SkillDiscoveryConfig`, `DiscoveredSkill`, and `discover_skills`.

- [x] **Step 2: Parse `SKILL.md`**

Split front matter between the first two `---` separators, parse YAML metadata, and use the remaining Markdown body as prompt content.

- [x] **Step 3: Verify GREEN**

Run:

```bash
cargo test -p ruse-skill skill_discovery
```

Expected: all skill discovery tests pass.

### Task 5: Update Progress and Verify

**Files:**
- Modify: `docs/plans/M07-03-skill-discovery.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Advance active focus**

Set active focus to `M07-04-mcp-skill-traceability`.

- [x] **Step 2: Run standard verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 3: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m07 skill discovery"
```

Expected: commit succeeds with source, tests, and documentation together.
