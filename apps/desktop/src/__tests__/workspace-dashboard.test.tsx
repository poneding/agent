import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { WorkspaceDashboard } from '../components/WorkspaceDashboard';
import type { DesktopDashboardSnapshot } from '../types';

function dashboard(
  overrides: Partial<DesktopDashboardSnapshot> = {},
): DesktopDashboardSnapshot {
  return {
    workspace_id: 'workspace-1',
    workspace_root_display: '/workspace/agent',
    session: {
      state: 'connected',
      session_id: 'session-1',
      conversation_id: 'conversation-1',
      pending_frame_count: 2,
    },
    provider: {
      provider_id: 'fake',
      model_id: 'fake-scripted',
      enabled: true,
      diagnostic_count: 1,
    },
    task_counts: {
      total: 5,
      active: 2,
      waiting_approval: 1,
      failed: 1,
      terminal: 2,
    },
    pending_approval_count: 1,
    diagnostics: [],
    ...overrides,
  };
}

describe('WorkspaceDashboard', () => {
  afterEach(() => cleanup());

  it('renders readiness, provider, task counts, and actions', () => {
    const openConversation = vi.fn();
    const openApprovals = vi.fn();

    render(
      <WorkspaceDashboard
        snapshot={dashboard()}
        onOpenConversation={openConversation}
        onOpenApprovals={openApprovals}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Dashboard' })).toBeInTheDocument();
    expect(screen.getByText('/workspace/agent')).toBeInTheDocument();
    expect(within(screen.getByLabelText('Session readiness')).getByText('connected')).toBeInTheDocument();
    expect(screen.getByText('fake-scripted')).toBeInTheDocument();
    expect(within(screen.getByLabelText('Task counts')).getByText('5')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Open conversation' }));
    fireEvent.click(screen.getByRole('button', { name: 'Review approvals' }));

    expect(openConversation).toHaveBeenCalledOnce();
    expect(openApprovals).toHaveBeenCalledOnce();
  });

  it('disables actions when disconnected or no approvals are pending', () => {
    render(
      <WorkspaceDashboard
        snapshot={dashboard({
          session: {
            state: 'disconnected',
            session_id: 'session-1',
            conversation_id: 'conversation-1',
            pending_frame_count: 0,
          },
          pending_approval_count: 0,
        })}
        onOpenConversation={vi.fn()}
        onOpenApprovals={vi.fn()}
      />,
    );

    expect(screen.getByRole('button', { name: 'Open conversation' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Review approvals' })).toBeDisabled();
  });

  it('renders bounded redacted diagnostics and loading state', () => {
    render(
      <WorkspaceDashboard
        snapshot={dashboard({
          diagnostics: [
            {
              severity: 'warning',
              code: '[redacted]',
              message: 'provider [redacted]',
              truncated: true,
            },
          ],
        })}
        onOpenConversation={vi.fn()}
        onOpenApprovals={vi.fn()}
      />,
    );

    expect(screen.getAllByText('[redacted]').length).toBeGreaterThan(0);
    expect(screen.getByText('truncated')).toBeInTheDocument();
    expect(screen.queryByText('sk-secret')).not.toBeInTheDocument();

    cleanup();

    render(
      <WorkspaceDashboard
        snapshot={null}
        onOpenConversation={vi.fn()}
        onOpenApprovals={vi.fn()}
      />,
    );
    expect(screen.getByText('Dashboard loading')).toBeInTheDocument();
  });
});
