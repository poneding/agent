import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { AppShell } from '../App';
import { createApprovalState } from '../state/approvals';
import type {
  DesktopBootstrap,
  DesktopConfigSnapshot,
  DesktopDashboardSnapshot,
  DesktopSessionSnapshot,
  DesktopSettings,
  DesktopWorkspaceSnapshot,
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
      task_id: 'task-1',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'running',
      summary: 'Desktop smoke task',
      updated_at: '2026-06-27T00:00:00Z',
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
    diagnostic_count: 0,
  },
  task_counts: {
    total: 1,
    active: 1,
    waiting_approval: 0,
    failed: 0,
    terminal: 0,
  },
  pending_approval_count: 0,
  diagnostics: [],
};

const traceTimeline: TraceTimeline = {
  conversation_id: 'conversation-1',
  items: [
    {
      seq: 1,
      kind: 'final',
      title: 'Final',
      summary: 'final response recorded',
      severity: 'info',
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
      },
    ],
    diagnostics: [],
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

describe('desktop MVP smoke navigation', () => {
  afterEach(() => cleanup());

  it('navigates conversation, tasks, approvals, trace, and settings views', () => {
    render(
      <AppShell
        bootstrap={bootstrap}
        bootstrapError={null}
        dashboardSnapshot={dashboardSnapshot}
        workspaceSnapshot={workspaceSnapshot}
        sessionSnapshot={sessionSnapshot}
        approvalState={createApprovalState()}
        traceTimeline={traceTimeline}
        configSnapshot={configSnapshot}
        settings={settings}
      />,
    );

    expect(screen.getByRole('main', { name: 'Dashboard workspace' })).toBeInTheDocument();
    expect(screen.getByText('/workspace/agent')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Open conversation' }));
    expect(screen.getByRole('main', { name: 'Conversation workspace' })).toBeInTheDocument();
    expect(screen.getByText('Main thread')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Tasks' }));
    const tasksWorkspace = screen.getByRole('main', { name: 'Tasks workspace' });
    expect(tasksWorkspace).toBeInTheDocument();
    expect(within(tasksWorkspace).getAllByText('Desktop smoke task').length).toBeGreaterThan(0);

    fireEvent.click(screen.getByRole('button', { name: 'Approvals' }));
    expect(screen.getByRole('main', { name: 'Approvals workspace' })).toBeInTheDocument();
    expect(within(screen.getByLabelText('Pending approvals')).getByText('No pending approvals')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Trace' }));
    expect(screen.getByRole('main', { name: 'Trace workspace' })).toBeInTheDocument();
    expect(screen.getByText('final response recorded')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    expect(screen.getByRole('main', { name: 'Settings workspace' })).toBeInTheDocument();
    expect(screen.getByText('fake-scripted')).toBeInTheDocument();
  });
});
