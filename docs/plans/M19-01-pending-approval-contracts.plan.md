# M19-01 Pending Approval Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add durable pending approval contracts and safe status transitions for policy `ask` decisions.

**Architecture:** `ruse-types` owns `ApprovalRequestId` as a durable typed id. `ruse-policy` owns approval records, response contracts, preview summaries, lease-scope clamping, and transition validation. No UI bridge, storage, CLI, or core resume behavior belongs in this slice.

**Tech Stack:** Rust 2024 edition, serde, uuid-backed typed ids, existing `ruse-policy` validation style, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M19-01-pending-approval-contracts.spec.md`
- Read: `docs/specs/M19-00-interactive-approval-technical-design.spec.md`
- Read: `crates/ruse-policy/src/request.rs`
- Read: `crates/ruse-policy/src/lease.rs`
- Read: `crates/ruse-types/src/ids.rs`

- [x] **Step 1: Review the spec and existing policy contracts**

Run:

```bash
sed -n '1,260p' docs/specs/M19-01-pending-approval-contracts.spec.md
sed -n '1,180p' crates/ruse-policy/src/request.rs
sed -n '1,180p' crates/ruse-types/src/ids.rs
```

Expected: the spec scopes M19-01 to typed contracts and transition validation only.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-policy/tests/pending_approval.rs`
- Modify: `crates/ruse-types/src/ids.rs`

- [x] **Step 1: Add pending approval contract tests**

Create `crates/ruse-policy/tests/pending_approval.rs` with tests covering construction, stable JSON, approval, denial, expiration, cancellation, consumption, stale responses, and lease clamping.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-policy --test pending_approval
```

Expected: compile fails because `PendingApproval`, `ApprovalRequestId`, `ApprovalStatus`, `ApprovalDecision`, `ApprovalResponse`, `ApprovalLeaseScope`, `ApprovalLeaseGrant`, `ApprovalPreviewSummary`, and `ApprovalResumeToken` are not implemented.

### Task 3: Add Typed Approval Id

**Files:**
- Modify: `crates/ruse-types/src/ids.rs`
- Modify: `crates/ruse-types/src/lib.rs`

- [x] **Step 1: Add `ApprovalRequestId`**

Add `uuid_id!(ApprovalRequestId);` beside the existing id newtypes in `crates/ruse-types/src/ids.rs`.

- [x] **Step 2: Cover typed id in RED/GREEN flow**

Run:

```bash
cargo test -p ruse-policy --test pending_approval
```

Expected: `ApprovalRequestId` is covered by the pending approval tests and exported through `ruse-types`.

### Task 4: Implement Approval Contracts

**Files:**
- Create: `crates/ruse-policy/src/approval.rs`
- Modify: `crates/ruse-policy/src/lib.rs`

- [x] **Step 1: Add public approval structs and enums**

Implement:

```rust
pub struct PendingApproval { ... }
pub enum ApprovalStatus { Pending, Approved, Denied, Expired, Cancelled, Consumed }
pub enum ApprovalDecision { Allow, Deny }
pub struct ApprovalResponse { ... }
pub struct ApprovalLeaseScope { ... }
pub struct ApprovalLeaseGrant { ... }
pub struct ApprovalPreviewSummary { ... }
pub struct ApprovalResumeToken { ... }
```

Derive `Clone`, `Debug`, `Eq`, `PartialEq`, `Serialize`, and `Deserialize` where appropriate.

- [x] **Step 2: Add constructors and validation**

Implement constructors that reject empty preview kind/resource/summary, empty response reasons, and empty resume token checkpoint labels if represented as strings.

- [x] **Step 3: Add status transition methods**

Implement:

```rust
impl PendingApproval {
    pub fn new(...) -> Result<Self, PolicyError>;
    pub fn approve(&mut self, response: ApprovalResponse) -> Result<ApprovalLeaseGrant, PolicyError>;
    pub fn deny(&mut self, response: ApprovalResponse) -> Result<(), PolicyError>;
    pub fn expire(&mut self, resolved_at: Timestamp) -> Result<(), PolicyError>;
    pub fn cancel(&mut self, resolved_at: Timestamp) -> Result<(), PolicyError>;
    pub fn consume(&mut self, resolved_at: Timestamp) -> Result<(), PolicyError>;
}
```

Only `Pending` records accept approve, deny, expire, and cancel. Only `Approved` records accept consume.

- [x] **Step 4: Export contracts**

Add `pub mod approval;` and `pub use approval::{...};` to `crates/ruse-policy/src/lib.rs`.

### Task 5: Verify M19-01

**Files:**
- Read: `crates/ruse-policy/tests/pending_approval.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-policy --test pending_approval
```

Expected: all pending approval tests pass.

- [x] **Step 2: Run policy crate tests**

Run:

```bash
cargo test -p ruse-policy
```

Expected: all `ruse-policy` tests pass.

- [x] **Step 3: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: formatting, clippy, and workspace tests pass.

### Task 6: Update Status

**Files:**
- Modify: `docs/plans/M19-01-pending-approval-contracts.plan.md`
- Modify: `docs/plans/M19-interactive-approval-safe-actions.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M19-01 complete**

Mark completed checkboxes in this plan, update M19 milestone plan Task 3 Step 1, and update milestone planning status to show M19-01 implementation complete.

- [x] **Step 2: Leave commit unchecked unless explicitly requested**

Do not commit by default in this repository session. The user has not requested a commit.
