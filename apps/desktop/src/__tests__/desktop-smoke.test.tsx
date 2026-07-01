import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { AppShell } from '../App';
import { createApprovalState } from '../state/approvals';
import type {
  ConversationTaskState,
  DesktopBootstrapDiagnostic,
  DesktopBootstrap,
  DesktopConfigSnapshot,
  DesktopDashboardSnapshot,
  DesktopDiffReview,
  DesktopSessionSnapshot,
  DesktopSettings,
  DesktopWorkspaceSnapshot,
  UiApprovalRequest,
  TraceTimeline,
} from '../types';

const bootstrap: DesktopBootstrap = {
  app_name: 'agent',
  app_version: '0.1.0',
  runtime_state: 'connected',
  route: 'dashboard',
  diagnostics: [],
};

const sessionSnapshot: DesktopSessionSnapshot = {
  session_id: 'session-1',
  workspace_id: 'workspace-1',
  conversation_id: 'conversation-1',
  state: 'connected',
  pending_frame_count: 0,
};

const workspaceSnapshot: DesktopWorkspaceSnapshot = {
  workspace_id: 'workspace-1',
  active_conversation_id: 'conversation-1',
  conversations: [
    {
      conversation_id: 'conversation-1',
      title: 'Main thread',
      last_activity_at: '2026-06-27T00:00:00Z',
      unread_count: 0,
    },
  ],
  tasks: [
    {
      task_id: 'task-63',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'running',
      summary: 'Apply Desktop smoke fixture update',
      updated_at: '2026-06-27T00:00:00Z',
      latest_run_summary: 'editing smoke fixture',
      artifact_count: 1,
    },
    {
      task_id: 'task-64',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'waiting_approval',
      summary: 'Review file-edit approval',
      updated_at: '2026-06-27T00:00:01Z',
      latest_run_summary: 'waiting on Desktop approval',
      artifact_count: 1,
    },
    {
      task_id: 'task-65',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'succeeded',
      summary: 'Run cargo xtask desktop-test',
      updated_at: '2026-06-27T00:00:02Z',
      latest_run_summary: 'Desktop product gate passed',
      artifact_count: 2,
    },
    {
      task_id: 'task-66',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'failed',
      summary: 'Capture recoverable log evidence',
      updated_at: '2026-06-27T00:00:03Z',
      latest_run_summary: 'recoverable warning captured',
      artifact_count: 0,
    },
  ],
};

const dashboardSnapshot: DesktopDashboardSnapshot = {
  workspace_id: 'workspace-1',
  workspace_root_display: '/workspace/agent',
  session: {
    state: 'connected',
    session_id: 'session-1',
    conversation_id: 'conversation-1',
    pending_frame_count: 0,
  },
  provider: {
    provider_id: 'fake',
    model_id: 'fake-scripted',
    enabled: true,
    diagnostic_count: 1,
  },
  task_counts: {
    total: 4,
    active: 1,
    waiting_approval: 1,
    failed: 1,
    terminal: 2,
  },
  pending_approval_count: 1,
  diagnostics: [],
};

const traceTimeline: TraceTimeline = {
  conversation_id: 'conversation-1',
  items: [
    {
      seq: 1,
      kind: 'context',
      title: 'Context',
      summary: 'workspace dashboard ready',
      severity: 'info',
    },
    {
      seq: 2,
      kind: 'permission',
      title: 'Permission',
      summary: 'approval requested and lease displayed',
      severity: 'info',
    },
    {
      seq: 3,
      kind: 'command_test',
      title: 'Command Test',
      summary: 'cargo xtask desktop-test passed',
      severity: 'info',
    },
    {
      seq: 4,
      kind: 'runtime_warning',
      title: 'Runtime Warning',
      summary: 'bounded warning [redacted]',
      severity: 'warning',
    },
    {
      seq: 5,
      kind: 'final',
      title: 'Final',
      summary: 'Desktop smoke fixture completed',
      severity: 'info',
    },
  ],
  logs: [
    {
      sequence: 1,
      severity: 'info',
      source: 'context',
      message: 'workspace dashboard ready',
      truncated: false,
    },
    {
      sequence: 2,
      severity: 'info',
      source: 'permission',
      message: 'approval requested and lease displayed',
      truncated: false,
    },
    {
      sequence: 3,
      severity: 'info',
      source: 'command_test',
      message: 'cargo xtask desktop-test passed',
      truncated: false,
    },
    {
      sequence: 4,
      severity: 'warning',
      source: 'runtime_warning',
      message: 'bounded warning [redacted]',
      truncated: false,
    },
  ],
};

const configSnapshot: DesktopConfigSnapshot = {
  snapshot: {
    selected_provider: 'fake',
    selected_model: 'fake-scripted',
    providers: [
      {
        id: 'fake',
        kind: 'fake',
        default_model: 'fake-scripted',
        enabled: true,
        capabilities: {
          streaming: true,
          tool_calls: true,
          structured_outputs: true,
          image_input: false,
          max_context_tokens: 8192,
          max_output_tokens: 2048,
        },
      },
    ],
    diagnostics: [
      {
        severity: 'info',
        code: 'desktop.provider.fake_ready',
        message: 'fake provider is ready for credential-free smoke',
        field_path: 'model.provider',
        source: 'm60-smoke',
      },
    ],
  },
};

const settings: DesktopSettings = {
  model: {
    provider: 'fake',
    model: 'fake-scripted',
  },
  policy: {
    default_decision: 'ask',
    require_approval_for_shell: true,
  },
  memory: {
    project_memory_enabled: false,
    long_term_memory_enabled: false,
  },
  mcp: {
    enabled: false,
    server_count: 0,
  },
  ui: {
    theme: 'system',
    show_trace_timeline: true,
  },
};

const settingsDiagnostics: DesktopBootstrapDiagnostic[] = [];

const approval: UiApprovalRequest = {
  request_id: 'approval-67',
  status: 'pending',
  action: 'write',
  resource_kind: 'file',
  resource: 'apps/desktop/src/__tests__/desktop-smoke.test.tsx',
  reason: 'policy requires review before applying Desktop smoke fixture changes',
  preview: {
    kind: 'file_edit',
    resource: 'apps/desktop/src/__tests__/desktop-smoke.test.tsx',
    summary: '+assert Desktop golden path evidence\n-[redacted]',
    content_hash: 'sha256:desktop-smoke',
    truncated: false,
    redacted: true,
  },
  max_duration_seconds: 120,
  expires_at: '2026-12-31T00:02:00Z',
};

const diffReview: DesktopDiffReview = {
  review_id: 'desktop-smoke-review',
  status: 'pending',
  selected_path: 'apps/desktop/src/__tests__/desktop-smoke.test.tsx',
  decision_enabled: true,
  disabled_reason: null,
  files: [
    {
      path: 'apps/desktop/src/__tests__/desktop-smoke.test.tsx',
      added_lines: 12,
      removed_lines: 2,
      summary: 'Expand Desktop smoke evidence assertions',
      hunks: [
        {
          old_start: 40,
          new_start: 40,
          truncated: false,
          lines: [
            {
              kind: 'context',
              old_line: 40,
              new_line: 40,
              content: 'render(<AppShell ... />)',
              truncated: false,
            },
            {
              kind: 'addition',
              old_line: null,
              new_line: 41,
              content: "+expect(screen.getByText('Desktop smoke fixture completed')).toBeInTheDocument()",
              truncated: false,
            },
          ],
        },
      ],
    },
  ],
};

const conversationState: ConversationTaskState = {
  workspaceId: 'workspace-1',
  conversations: workspaceSnapshot.conversations,
  tasks: workspaceSnapshot.tasks,
  transcript: [
    {
      id: 'entry-1',
      kind: 'user',
      content: 'Update the Desktop smoke fixture and run tests',
    },
    {
      id: 'entry-2',
      kind: 'progress',
      content: 'prepared file edit preview',
    },
    {
      id: 'entry-3',
      kind: 'assistant',
      content: 'Desktop smoke fixture completed with tests passing.',
    },
  ],
  activeConversationId: 'conversation-1',
  activeTaskId: 'task-64',
  composerText: '',
  currentSubmission: {
    command_id: 'command-1',
    conversation_id: 'conversation-1',
    prompt_preview: 'Update the Desktop smoke fixture and run cargo xtask desktop-test',
    status: 'completed',
    ack: {
      command_id: 'command-1',
      accepted: true,
      pending_frame_count: 0,
    },
    error: null,
    truncated: false,
  },
  diagnostics: [],
  pendingFrameCount: 0,
};

describe('desktop MVP smoke navigation', () => {
  afterEach(() => cleanup());

  it('navigates the M56-shaped Desktop golden path without exposing secrets', () => {
    const onSubmitPrompt = vi.fn();
    const onAllowApproval = vi.fn();
    const onDenyApproval = vi.fn();
    const approvalState = {
      ...createApprovalState(),
      pending: [approval],
      selectedRequestId: approval.request_id,
    };

    render(
      <AppShell
        bootstrap={bootstrap}
        bootstrapError={null}
        dashboardSnapshot={dashboardSnapshot}
        workspaceSnapshot={workspaceSnapshot}
        conversationState={conversationState}
        sessionSnapshot={sessionSnapshot}
        approvalState={approvalState}
        onSubmitPrompt={onSubmitPrompt}
        onAllowApproval={onAllowApproval}
        onDenyApproval={onDenyApproval}
        traceTimeline={traceTimeline}
        configSnapshot={configSnapshot}
        settings={settings}
        settingsDiagnostics={settingsDiagnostics}
        diffReview={diffReview}
      />,
    );

    expect(screen.getByRole('main', { name: 'Dashboard workspace' })).toBeInTheDocument();
    expect(screen.getByText('/workspace/agent')).toBeInTheDocument();
    expect(screen.getAllByText('connected').length).toBeGreaterThan(0);
    expect(screen.getByRole('button', { name: 'Review approvals' })).toBeEnabled();

    fireEvent.click(screen.getByRole('button', { name: 'Open conversation' }));
    const conversationWorkspace = screen.getByRole('main', { name: 'Conversation workspace' });
    expect(conversationWorkspace).toBeInTheDocument();
    expect(within(conversationWorkspace).getByText('Main thread')).toBeInTheDocument();
    expect(within(conversationWorkspace).getByText('Desktop smoke fixture completed with tests passing.')).toBeInTheDocument();
    expect(within(conversationWorkspace).getByText('completed')).toBeInTheDocument();
    fireEvent.change(within(conversationWorkspace).getByLabelText('Message composer'), {
      target: { value: 'Run the Desktop product gate' },
    });
    fireEvent.click(within(conversationWorkspace).getByRole('button', { name: 'Send' }));
    expect(onSubmitPrompt).toHaveBeenCalledWith('Run the Desktop product gate');

    fireEvent.click(screen.getByRole('button', { name: 'Tasks' }));
    const tasksWorkspace = screen.getByRole('main', { name: 'Tasks workspace' });
    expect(tasksWorkspace).toBeInTheDocument();
    expect(within(tasksWorkspace).getAllByText('Review file-edit approval').length).toBeGreaterThan(0);
    expect(within(tasksWorkspace).getAllByText('waiting_approval').length).toBeGreaterThan(0);
    expect(within(tasksWorkspace).getByText('Run cargo xtask desktop-test')).toBeInTheDocument();
    expect(within(tasksWorkspace).getByText('succeeded')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Approvals' }));
    const approvalsWorkspace = screen.getByRole('main', { name: 'Approvals workspace' });
    expect(approvalsWorkspace).toBeInTheDocument();
    expect(within(approvalsWorkspace).getByText('file_edit')).toBeInTheDocument();
    expect(within(approvalsWorkspace).getByText('redacted')).toBeInTheDocument();
    expect(within(approvalsWorkspace).getByLabelText('Visual diff review')).toBeInTheDocument();
    expect(within(approvalsWorkspace).getByText('Expand Desktop smoke evidence assertions')).toBeInTheDocument();
    fireEvent.click(within(approvalsWorkspace).getByRole('button', { name: 'Allow' }));
    expect(onAllowApproval).toHaveBeenCalledWith('approval-67');
    fireEvent.change(within(approvalsWorkspace).getByLabelText('Deny reason'), {
      target: { value: 'not safe enough' },
    });
    fireEvent.click(within(approvalsWorkspace).getByRole('button', { name: 'Deny' }));
    expect(onDenyApproval).toHaveBeenCalledWith('approval-67', 'not safe enough');

    fireEvent.click(screen.getByRole('button', { name: 'Trace' }));
    const traceWorkspace = screen.getByRole('main', { name: 'Trace workspace' });
    expect(traceWorkspace).toBeInTheDocument();
    expect(within(traceWorkspace).getAllByText('cargo xtask desktop-test passed').length).toBeGreaterThan(0);
    expect(within(traceWorkspace).getByText('Runtime logs')).toBeInTheDocument();
    expect(within(traceWorkspace).getAllByText('bounded warning [redacted]').length).toBeGreaterThan(0);

    fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    const settingsWorkspace = screen.getByRole('main', { name: 'Settings workspace' });
    expect(settingsWorkspace).toBeInTheDocument();
    expect(within(settingsWorkspace).getByText('fake-scripted')).toBeInTheDocument();
    expect(within(settingsWorkspace).getByText('streaming yes / tools yes / context 8192')).toBeInTheDocument();
    expect(within(settingsWorkspace).getByText('desktop.provider.fake_ready')).toBeInTheDocument();

    for (const forbidden of ['sk-secret', 'api_key', 'token', 'password']) {
      expect(screen.queryByText(forbidden)).not.toBeInTheDocument();
    }
  });
});
