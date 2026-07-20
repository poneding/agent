# M11-02 Process Plugin Host Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a typed process plugin host and fixture process tests.

**Architecture:** Keep process protocol and host errors in `ruse-plugin`. Do not register tools or evaluate permissions in this slice.

**Tech Stack:** Rust 2024 edition, `serde`, `serde_json`, `std::process`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M11-00-plugin-runtime-technical-design.spec.md`
- Read: `docs/specs/M11-02-process-plugin-host.spec.md`
- Read: `crates/ruse-plugin/src/manifest.rs`

- [x] **Step 1: Review process protocol**

Run:

```bash
sed -n '1,260p' docs/specs/M11-02-process-plugin-host.spec.md
```

Expected: the process request/response protocol and failure modes are explicit.

### Task 2: Write RED Process Host Tests

**Files:**
- Create: `crates/ruse-plugin/tests/process_host.rs`

- [x] **Step 1: Add failing tests**

Add tests:

- `process_host_invokes_fixture_tool`
- `process_host_maps_plugin_error_response`
- `process_host_rejects_invalid_json_response`
- `process_host_rejects_mismatched_call_id`
- `process_host_rejects_missing_entrypoint`
- `process_host_rejects_oversized_stdout`
- `process_host_times_out_slow_process`

Use serialized Unix fixture scripts as the process entrypoint. Each fixture reads stdin and emits one JSON response or a controlled failure.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-plugin process_host
```

Expected: tests fail because `host` and `process` modules do not exist.

### Task 3: Implement Process Contracts

**Files:**
- Modify: `crates/ruse-plugin/src/lib.rs`
- Create: `crates/ruse-plugin/src/host.rs`
- Create: `crates/ruse-plugin/src/process.rs`
- Modify: `crates/ruse-plugin/src/error.rs`

- [x] **Step 1: Add protocol types**

Define request, response, kind, and result enums using serde with `snake_case` names.

- [x] **Step 2: Add host error variants**

Add `PluginProcessError` variants for invalid config, spawn, exit, timeout, output too large, and protocol failures.

- [x] **Step 3: Add process host**

Implement `PluginProcessHost::invoke_tool(config, request)` so it:

- serializes a request to JSON
- writes the request as one newline-terminated JSON frame
- passes JSON to the process
- reads stdout
- checks output limit
- parses response JSON
- verifies protocol version and call id
- returns output JSON or typed host error

### Task 4: Verify GREEN

**Files:**
- Read: `crates/ruse-plugin/tests/process_host.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-plugin process_host
```

Expected: process host tests pass.

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
- Modify: `docs/plans/M11-02-process-plugin-host.plan.md`

- [x] **Step 1: Mark plan items complete**

Check off completed steps in this plan.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m11 process plugin host"
```

Expected: commit succeeds.
