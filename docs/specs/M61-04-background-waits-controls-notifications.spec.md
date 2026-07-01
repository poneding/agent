# M61-04 Background Wait Controls and Notifications Spec

## Purpose

Make background approval waits, pause/resume/cancel controls, and latest task notifications visible as typed evidence across scheduler, core, bridge, CLI, and Desktop task surfaces.

## Product Context

M61-01 added shared plan/todo contracts, M61-02 exposed CLI/JSON commands, and M61-03 rendered plan/task state in TUI and Desktop. Existing scheduler and background runner code already supports leases, approval waits, pause/resume/cancel, and task notifications. M61-04 connects those runtime facts to task summaries so users can answer: what is this background task waiting on, which approval id is involved, and what control action most recently changed it?

## Scope

This slice delivers:

- scheduler-owned wait/control evidence on task records and command task summaries
- persisted waiting approval ids for scheduler queue tasks that enter `waiting_approval`
- typed latest notification summaries for queued, waiting approval, resumed, cancelled, paused, completed, failed, and schedule-updated transitions where the scheduler already owns the state change
- UI bridge task summary fields for waiting approval id, latest notification kind, latest notification time, and latest notification approval id
- CLI text/JSON task command output that shows latest notification/control evidence without exposing raw prompts or secrets
- Desktop Rust and TypeScript task models that carry and render the wait/control evidence
- focused tests for scheduler, background approval waits, UI bridge, CLI, and Desktop conversion/rendering

Out of scope:

- OS-level notifications, Desktop native notification permission prompts, or background daemons
- new approval policy semantics or bypasses around existing policy/scheduler paths
- task recovery, crash resume, fork/replay UX, or failed-command retry UX beyond existing scheduler state
- full task artifact closeout and workflow docs, which remain in M61-05

## Public Contracts

Scheduler task records and command summaries add optional serde fields:

```rust
pub struct TaskRun {
    pub waiting_approval_id: Option<ApprovalRequestId>,
    pub latest_notification: Option<TaskNotificationSummary>,
}

pub struct BackgroundAgentTask {
    pub waiting_approval_id: Option<ApprovalRequestId>,
    pub latest_notification: Option<TaskNotificationSummary>,
}

pub struct TaskNotificationSummary {
    pub kind: TaskNotificationKind,
    pub summary: String,
    pub occurred_at: Timestamp,
    pub approval_id: Option<ApprovalRequestId>,
}
```

UI task summaries expose additive fields while preserving the existing `latest_notification: Option<String>` summary string:

```rust
pub struct UiTaskSummary {
    pub waiting_approval: bool,
    pub waiting_approval_id: Option<ApprovalRequestId>,
    pub latest_notification: Option<String>,
    pub latest_notification_kind: Option<String>,
    pub latest_notification_at: Option<Timestamp>,
    pub latest_notification_approval_id: Option<ApprovalRequestId>,
}
```

All fields are optional/defaulted for backward-compatible deserialization.

## State and Failure Handling

- `wait_for_approval` records the approval id on the task run, clears active leases, and stores a latest `waiting_approval` notification summary.
- `resume` and `cancel` clear `waiting_approval_id` and record `resumed` or `cancelled` evidence. Approval resolution still emits notification sink records with the approval id.
- `pause`, `complete`, and terminal failures record latest notification evidence where state transition succeeds.
- Command-service pause/resume/cancel/schedule updates record latest control evidence on `BackgroundAgentTask`.
- Empty or secret-like summaries are normalized, bounded, and redacted in scheduler-owned constructors before they reach UI surfaces.
- Invalid transitions continue to fail through existing `SchedulerError` paths and do not mutate evidence fields.

## Security and Privacy

- Notification summaries and control reasons are bounded and redacted with the scheduler redaction helper.
- Approval ids and task ids may be surfaced; raw tool input, provider credentials, environment secrets, and unbounded command output must not be exposed.
- CLI, Desktop Rust, and TypeScript only render bridge/runtime fields. They do not own task lifecycle decisions.
- Existing scheduler and policy approval paths remain the only route for background wait resolution and task controls.

## Testing Strategy

- `agent-scheduler` tests cover command-service control evidence, queue approval id persistence, and redaction.
- `agent-core` background approval tests cover wait/resume/cancel outcomes and notification ids from the runner.
- `agent-ui-bridge` tests cover new task summary fields, JSON shape, defaulted backward compatibility, and redaction.
- `agent-cli` tests cover text lifecycle output and JSON snapshot evidence.
- `agent-desktop` tests cover Desktop task item and monitor conversion of approval ids and notification metadata.
- Focused verification runs fmt, clippy for touched crates, relevant Rust tests, and `git diff --check`.

## Acceptance Criteria

- Background approval waits expose the approval id through scheduler state and UI task summaries.
- Pause/resume/cancel and schedule updates expose latest control evidence in typed scheduler, CLI JSON, and Desktop-visible task summaries.
- Existing task notifications still emit UI events with approval ids.
- All new fields are serde-friendly, defaulted where additive, bounded, redacted, and covered by focused tests.
- Runtime ownership stays in scheduler/core; presentation layers only render typed evidence.
