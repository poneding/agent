# MX02-04 Inspector Panels for Changes, Approvals, Tasks, and Artifacts Spec

## Purpose

Productize the right-side Desktop inspector so users can inspect context, changes, approvals, tasks, artifacts, and trace evidence without leaving the active thread.

## Product Context

M60 created separate approval, diff, task, settings, and trace routes. MX02-04 brings those surfaces into a compact tabbed inspector that supports the MX02 shell while preserving typed Desktop/runtime contracts.

## Scope

This spec delivers:

- Context panel placeholder for selected workspace/thread context and pinned context summaries
- Changes panel with file list, hunk preview, staged/unstaged badges, stage/revert placeholders, and inline comment placeholder
- Approvals panel with approve once/session/deny, risk rationale, sandbox/network scope, expiration, and auto-review status placeholder
- Tasks panel with plan/todo progress, active todo, waiting approval, notifications, and artifact links
- Artifacts panel with typed metadata previews for files, docs, sheets, slides, images, and other files without requiring full rendering
- Trace panel with compact runtime timeline

Owned by other specs:

- real staging/revert side effects remain future tool/policy work
- full artifact renderers are outside this first inspector slice
- recovery-specific views remain in M62 unless later wired into the Trace/Tasks tabs

## Architecture

Inspector panels render `agent-desktop` view models and `agent-ui-bridge` frames. React owns active tab and panel expansion state only. Approval decisions still route through existing Tauri bridge commands and policy-backed runtime paths.

## File Layout

```text
crates/agent-desktop/src/
  diff.rs
  permission.rs
  task.rs
  trace.rs
apps/desktop/src/
  components/
  state/
  types.ts
  __tests__/approval-diff-ui.test.tsx
  __tests__/conversation-task-ui.test.tsx
  __tests__/desktop-smoke.test.tsx
```

## Public Contracts

Potential Desktop additions:

```rust
pub struct DesktopInspectorSnapshot {
    pub context: DesktopContextInspector,
    pub changes: DesktopChangesInspector,
    pub approvals: Vec<DesktopApprovalReview>,
    pub tasks: DesktopTaskMonitor,
    pub artifacts: Vec<DesktopArtifactPreview>,
    pub trace: TraceTimeline,
}

pub struct DesktopArtifactPreview {
    pub artifact_id: String,
    pub display_name: String,
    pub media_type: String,
    pub size_label: String,
    pub source_label: String,
    pub redacted: bool,
}
```

## State and Failure Handling

- Empty panels show explicit empty states.
- Missing artifact metadata creates diagnostics instead of hiding the artifact.
- Expired/resolved approvals disable action controls.
- Stage/revert/comment controls are placeholders until typed commands exist.
- Trace timeline is compact and bounded.

## Security and Privacy

- Diffs, command output, artifact metadata, trace entries, approval rationale, and notifications are bounded and redacted before React.
- Inspector controls cannot directly mutate files, Git state, or approvals outside typed bridge commands.

## Testing Strategy

- `agent-desktop` tests for diff/approval/task/artifact/trace view models and redaction
- Vitest approval/diff, conversation/task, and desktop-smoke tests for inspector tabs and panel states
- Build check for TypeScript/Vite

## Acceptance Criteria

- Right inspector tabs are usable from the active thread without route switching.
- Changes, approvals, tasks, artifacts, and trace panels render typed information safely.
- Placeholder actions are visually distinct from implemented policy-backed actions.
- No raw secrets or unbounded output reach React state.

## Completion Signal

MX02-04 is complete when focused inspector Rust/frontend checks and build pass.
