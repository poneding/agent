import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { AppShell, BootstrapError } from '../App';
import type { ConversationTaskState, DesktopBootstrap, DesktopSessionSnapshot } from '../types';

const bootstrap: DesktopBootstrap = {
  app_name: 'agent',
  app_version: '0.1.0',
  runtime_state: 'not_started',
  route: 'dashboard',
  diagnostics: [
    {
      severity: 'warning',
      code: 'desktop.bootstrap',
      message: 'using fixture bootstrap',
    },
  ],
};

const sessionSnapshot: DesktopSessionSnapshot = {
  session_id: 'session-1',
  workspace_id: 'workspace-1',
  conversation_id: 'conversation-1',
  state: 'connected',
  pending_frame_count: 0,
};

const conversationState: ConversationTaskState = {
  workspaceId: 'workspace-1',
  activeConversationId: 'conversation-1',
  activeTaskId: null,
  composerText: '',
  conversations: [
    {
      conversation_id: 'conversation-1',
      title: 'Implement desktop shell',
      last_activity_at: '2026-07-02T00:00:00Z',
      unread_count: 1,
    },
  ],
  currentSubmission: null,
  diagnostics: [],
  pendingFrameCount: 0,
  tasks: [],
  transcript: [
    {
      id: 'entry-1',
      kind: 'assistant',
      content: 'Ready to work',
    },
  ],
};

afterEach(() => cleanup());

describe('AppShell', () => {
  it('renders the MX02 workspace shell regions from typed state', () => {
    render(
      <AppShell
        bootstrap={bootstrap}
        bootstrapError={null}
        conversationState={conversationState}
        sessionSnapshot={sessionSnapshot}
      />,
    );

    expect(
      screen.getByRole('navigation', { name: 'Workspace and thread navigation' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('main', { name: 'Active thread workspace' })).toBeInTheDocument();
    expect(screen.getByRole('complementary', { name: 'Inspector' })).toBeInTheDocument();
    expect(screen.getByRole('tablist', { name: 'Inspector tabs' })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: 'Recent threads' })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: 'Pinned threads' })).toBeInTheDocument();
    expect(screen.getByRole('region', { name: 'Archived threads' })).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: 'Search threads' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '+ New Thread' })).toBeInTheDocument();
    expect(screen.getByText('Implement desktop shell')).toBeInTheDocument();
    expect(screen.getByText('Ready to work')).toBeInTheDocument();
    expect(screen.getAllByText('agent')).toHaveLength(2);
    expect(screen.getAllByText('0.1.0')).toHaveLength(2);
    expect(screen.getAllByText('not_started')).toHaveLength(2);
    expect(screen.getByText('desktop.bootstrap')).toBeInTheDocument();
  });

  it('switches inspector tabs and toggles terminal and command palette controls', () => {
    render(
      <AppShell
        bootstrap={bootstrap}
        bootstrapError={null}
        conversationState={conversationState}
        sessionSnapshot={sessionSnapshot}
      />,
    );

    fireEvent.click(screen.getByRole('tab', { name: 'Changes' }));
    expect(screen.getByRole('tabpanel', { name: 'Changes' })).toHaveTextContent(
      'No changes to review',
    );

    fireEvent.click(screen.getByRole('button', { name: 'Toggle terminal' }));
    expect(screen.getByRole('region', { name: 'Integrated terminal' })).toHaveTextContent(
      'No terminal session',
    );

    fireEvent.click(screen.getByRole('button', { name: 'Open command palette' }));
    expect(screen.getByRole('dialog', { name: 'Command palette' })).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: 'Command search' })).toBeInTheDocument();
  });

  it('supports keyboard-first shell shortcuts', () => {
    const { container } = render(
      <AppShell
        bootstrap={bootstrap}
        bootstrapError={null}
        conversationState={conversationState}
        sessionSnapshot={sessionSnapshot}
      />,
    );
    const shell = container.querySelector('.desktop-shell');
    expect(shell).not.toBeNull();

    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    expect(screen.getByRole('dialog', { name: 'Command palette' })).toBeInTheDocument();

    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    expect(screen.queryByRole('dialog', { name: 'Command palette' })).not.toBeInTheDocument();

    fireEvent.keyDown(window, { key: 'f', metaKey: true });
    expect(screen.getByRole('textbox', { name: 'Find in thread' })).toBeInTheDocument();

    fireEvent.keyDown(window, { key: 'j', metaKey: true });
    expect(screen.getByRole('region', { name: 'Integrated terminal' })).toBeInTheDocument();

    fireEvent.keyDown(window, { key: 'b', metaKey: true });
    expect(shell).toHaveClass('sidebar-collapsed');

    fireEvent.keyDown(window, { key: 'i', metaKey: true });
    expect(shell).toHaveClass('inspector-collapsed');

    fireEvent.keyDown(window, { key: 'n', metaKey: true });
    expect(screen.getByRole('status')).toHaveTextContent('New thread shortcut captured');
  });

  it('renders bootstrap errors as recoverable diagnostics', () => {
    render(<BootstrapError message="bootstrap failed" />);

    expect(screen.getByRole('alert')).toHaveTextContent('bootstrap failed');
  });
});
