# M02-02 Policy Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `agent-policy` with permission requests, allow/deny/ask decisions, and leases.

**Architecture:** `agent-policy` depends on `agent-types` for actor, timestamp, and correlation contracts. It stays independent from `agent-tool` so policy can evaluate any capability source.

**Tech Stack:** Rust 2024 edition, `agent-types`, `serde`, `thiserror`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Policy Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/agent-policy/Cargo.toml`
- Create: `crates/agent-policy/src/lib.rs`

- [x] **Step 1: Add workspace member and manifest**

Dependencies:

```toml
[dependencies]
agent-types = { path = "../agent-types" }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
```

- [x] **Step 2: Export modules**

Export `capability`, `decision`, `engine`, `error`, `lease`, and `request`.

### Task 2: Write Failing Policy Tests

**Files:**
- Create: `crates/agent-policy/tests/policy_engine.rs`

- [x] **Step 1: Add tests**

Tests must prove:

- default static policy returns `Ask`
- explicit allow returns `Allow`
- explicit deny returns `Deny`
- allow can issue a lease linked to the request
- request and decision serialize to stable JSON

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-policy policy_engine
```

Expected: tests fail before contracts exist.

### Task 3: Implement Policy Contracts

**Files:**
- Create: `crates/agent-policy/src/capability.rs`
- Create: `crates/agent-policy/src/decision.rs`
- Create: `crates/agent-policy/src/engine.rs`
- Create: `crates/agent-policy/src/error.rs`
- Create: `crates/agent-policy/src/lease.rs`
- Create: `crates/agent-policy/src/request.rs`

- [x] **Step 1: Implement data contracts**

Implement `CapabilityAction`, `ResourceKind`, `ResourceScope`, `PermissionRequest`, `PolicyDecision`, and `PermissionLease`.

- [x] **Step 2: Implement static engine**

Implement `StaticPolicyEngine` with explicit rules and default `Ask`.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p agent-policy policy_engine
```

Expected: focused tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M02-02-policy-engine.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M02-02-policy-engine` to `M02-03-read-only-builtins`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m02-02 policy engine"
```
