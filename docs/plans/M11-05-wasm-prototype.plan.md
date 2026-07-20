# M11-05 WASM Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a constrained WASM plugin host prototype with validation-only capabilities.

**Architecture:** Keep WASM prototype code in `ruse-plugin::wasm`; do not introduce filesystem, network, command, or secret host calls.

**Tech Stack:** Rust 2024 edition, byte validation, serde-compatible errors where needed, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M11-05-wasm-prototype.spec.md`
- Read: `docs/specs/M11-00-plugin-runtime-technical-design.spec.md`

- [x] **Step 1: Review WASM constraints**

Run:

```bash
sed -n '1,240p' docs/specs/M11-05-wasm-prototype.spec.md
```

Expected: the prototype rejects privileged host behavior.

### Task 2: Write RED WASM Tests

**Files:**
- Create: `crates/ruse-plugin/tests/wasm_host.rs`

- [x] **Step 1: Add failing tests**

Add tests:

- `wasm_host_accepts_minimal_module_header`
- `wasm_host_rejects_invalid_magic`
- `wasm_host_rejects_oversized_module`
- `wasm_host_rejects_manifest_without_entrypoint`
- `wasm_host_rejects_unsupported_host_call`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-plugin wasm_host
```

Expected: tests fail because `wasm` module does not exist.

### Task 3: Implement WASM Prototype

**Files:**
- Modify: `crates/ruse-plugin/src/lib.rs`
- Create: `crates/ruse-plugin/src/wasm.rs`
- Modify: `crates/ruse-plugin/src/error.rs`

- [x] **Step 1: Add WASM config and module types**

Implement `PluginWasmConfig`, `PluginWasmModule`, and `PluginWasmHost`.

- [x] **Step 2: Add module validation**

Validate:

- magic bytes `\0asm`
- version bytes `1, 0, 0, 0`
- module byte length below `max_module_bytes`

- [x] **Step 3: Add unsupported host call guard**

Expose a method that rejects named host calls with `PluginWasmError::UnsupportedHostCall`.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/ruse-plugin/tests/wasm_host.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-plugin wasm_host
```

Expected: WASM tests pass.

- [x] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Complete M11 And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M11-05-wasm-prototype.plan.md`

- [x] **Step 1: Mark M11 complete**

Update milestone status and set active focus to M12 planning.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m11 plugin runtime"
```

Expected: commit succeeds.
