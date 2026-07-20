# M16-04 CLI Prompt Inspection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe CLI commands for prompt profile listing and effective prompt inspection.

**Architecture:** `ruse-cli` owns parsing and rendering. It uses `ruse-context` prompt runtime contracts and existing config evaluation, but it does not invoke model providers or persist runtime events.

**Tech Stack:** Rust 2024 edition, existing CLI parser, serde JSON frames, ruse-context prompt runtime, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M16-04-cli-prompt-inspection.spec.md`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-cli/tests/cli_modes.rs`
- Read: `crates/ruse-cli/tests/json_mode.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M16-04-cli-prompt-inspection.spec.md
```

Expected: the spec defines prompt inspection CLI behavior without model invocation.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-cli/tests/prompt_inspection.rs`
- Modify: `crates/ruse-cli/tests/cli_modes.rs`
- Modify: `crates/ruse-cli/tests/json_mode.rs`

- [x] **Step 1: Add CLI parser tests**

Add tests named:

- `cli_modes_parse_prompt_profiles`
- `cli_modes_parse_prompt_inspect`

- [x] **Step 2: Add text output tests**

Add tests named:

- `prompt_inspection_profiles_lists_default_profile`
- `prompt_inspection_text_renders_profile_and_layers`
- `prompt_inspection_text_redacts_hidden_and_secret_like_content`

- [x] **Step 3: Add JSON output tests**

Add tests named:

- `prompt_inspection_json_renders_parseable_frames`
- `prompt_inspection_json_redacts_hidden_and_secret_like_content`

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p ruse-cli --test prompt_inspection
cargo test -p ruse-cli --test cli_modes
cargo test -p ruse-cli --test json_mode
```

Expected: tests fail because prompt subcommands and renderers are not implemented.

### Task 3: Implement CLI Prompt Inspection

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/src/lib.rs` if exports are needed

- [x] **Step 1: Add command variants**

Add `CliCommand::PromptProfiles` and `CliCommand::PromptInspect`.

- [x] **Step 2: Parse prompt subcommands**

Parse:

- `prompt profiles`
- `prompt inspect`

- [x] **Step 3: Render text inspection**

Render default profile id, display name, layer ids, token estimates, diagnostics, and bundle hash without model calls.

- [x] **Step 4: Render JSON inspection**

Emit newline-delimited JSON frames and a final done frame. Keep hidden and secret-like content out of payloads.

### Task 4: Verify And Update Status

**Files:**
- Modify: `docs/plans/M16-04-cli-prompt-inspection.plan.md`
- Modify: `docs/plans/M16-prompt-runtime-dynamic-management.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-cli --test prompt_inspection
cargo test -p ruse-cli --test cli_modes
cargo test -p ruse-cli --test json_mode
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: formatting, clippy, and workspace tests pass.

- [x] **Step 3: Mark M16-04 complete**

Update M16 planning status so active work advances to M16-05.

- [ ] **Step 4: Commit**

Run:

```bash
git add crates/ruse-cli docs
git commit -m "feat: add m16 cli prompt inspection"
```

Expected: commit succeeds if this slice is landed independently.
