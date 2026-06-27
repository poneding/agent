import { useEffect, useMemo, useState } from 'react';

import { desktopBootstrap, loadWorkspaceSnapshot, resolveApproval } from './api';
import { ApprovalDetail } from './components/ApprovalDetail';
import { ApprovalQueue } from './components/ApprovalQueue';
import { Composer } from './components/Composer';
import { ConversationList } from './components/ConversationList';
import { TaskDetail } from './components/TaskDetail';
import { TaskList } from './components/TaskList';
import { TranscriptView } from './components/TranscriptView';
import { sendUiCommand, startSession } from './bridge';
import {
  createConversationTaskState,
  createStartTurnEnvelope,
} from './state/conversation';
import { createApprovalState } from './state/approvals';
import { selectTask } from './state/tasks';
import type {
  ApprovalUiState,
  DesktopBootstrap,
  DesktopSessionSnapshot,
  DesktopWorkspaceSnapshot,
} from './types';

export function BootstrapError({ message }: { message: string }) {
  return (
    <section className="diagnostic diagnostic-error" role="alert">
      <strong>Bootstrap failed</strong>
      <span>{message}</span>
    </section>
  );
}

export function AppShell({
  bootstrap,
  bootstrapError,
  workspaceSnapshot,
  sessionSnapshot,
  onSubmitPrompt = () => undefined,
  approvalState = createApprovalState(),
  onAllowApproval = () => undefined,
  onDenyApproval = () => undefined,
}: {
  bootstrap: DesktopBootstrap | null;
  bootstrapError: string | null;
  workspaceSnapshot?: DesktopWorkspaceSnapshot | null;
  sessionSnapshot?: DesktopSessionSnapshot | null;
  onSubmitPrompt?: (prompt: string) => void;
  approvalState?: ApprovalUiState;
  onAllowApproval?: (requestId: string) => void;
  onDenyApproval?: (requestId: string, reason: string) => void;
}) {
  const diagnostics = bootstrap?.diagnostics ?? [];
  const conversationTaskState = useMemo(
    () =>
      createConversationTaskState(
        workspaceSnapshot ?? {
          workspace_id: 'loading',
          active_conversation_id: null,
          conversations: [],
          tasks: [],
        },
      ),
    [workspaceSnapshot],
  );
  const selectedTask = selectTask(
    conversationTaskState.tasks,
    conversationTaskState.activeTaskId,
  );
  const selectedApproval =
    approvalState.pending.find(
      (approval) => approval.request_id === approvalState.selectedRequestId,
    ) ?? null;
  const composerDisabled =
    sessionSnapshot?.state !== 'connected' || conversationTaskState.activeConversationId === null;

  return (
    <div className="desktop-shell">
      <nav className="left-rail" aria-label="Desktop navigation">
        <div className="brand">
          <span className="brand-name">{bootstrap?.app_name ?? 'agent'}</span>
          <span className="brand-version">{bootstrap?.app_version ?? 'starting'}</span>
        </div>
        <button className="rail-item active" type="button">
          Conversations
        </button>
        <button className="rail-item" type="button">
          Tasks
        </button>
        <button className="rail-item" type="button">
          Trace
        </button>
        <button className="rail-item" type="button">
          Settings
        </button>
      </nav>

      <main className="workspace" aria-label="Conversation workspace">
        <section className="transcript-band">
          <h1>Conversation</h1>
          <div className="conversation-layout">
            <ConversationList
              activeConversationId={conversationTaskState.activeConversationId}
              conversations={conversationTaskState.conversations}
              onSelect={() => undefined}
            />
            <TranscriptView entries={conversationTaskState.transcript} />
          </div>
        </section>
        <Composer disabled={composerDisabled} onSubmitPrompt={onSubmitPrompt} />
      </main>

      <aside className="context-panel" aria-label="Context panel">
        {bootstrapError ? <BootstrapError message={bootstrapError} /> : null}
        <section className="panel-section">
          <h2>Runtime</h2>
          <dl>
            <dt>State</dt>
            <dd>{bootstrap?.runtime_state ?? 'loading'}</dd>
            <dt>Route</dt>
            <dd>{bootstrap?.route ?? 'conversation'}</dd>
          </dl>
        </section>
        <section className="panel-section">
          <h2>Tasks</h2>
          <TaskList
            activeTaskId={conversationTaskState.activeTaskId}
            tasks={conversationTaskState.tasks}
            onSelect={() => undefined}
          />
          <TaskDetail task={selectedTask} />
        </section>
        <section className="panel-section">
          <h2>Approvals</h2>
          <ApprovalQueue
            approvals={approvalState.pending}
            selectedRequestId={approvalState.selectedRequestId}
            onSelect={() => undefined}
          />
          <ApprovalDetail
            approval={selectedApproval}
            onAllow={onAllowApproval}
            onDeny={onDenyApproval}
          />
        </section>
        <section className="panel-section">
          <h2>Diagnostics</h2>
          {diagnostics.length === 0 ? (
            <p>No diagnostics</p>
          ) : (
            <ul>
              {diagnostics.map((diagnostic) => (
                <li key={diagnostic.code}>
                  <strong>{diagnostic.code}</strong>
                  <span>{diagnostic.message}</span>
                </li>
              ))}
            </ul>
          )}
        </section>
      </aside>

      <footer className="status-bar">
        <span>{bootstrap?.app_name ?? 'agent'}</span>
        <span>{bootstrap?.app_version ?? '0.1.0'}</span>
        <span>{bootstrap?.runtime_state ?? 'loading'}</span>
      </footer>
    </div>
  );
}

export default function App() {
  const [bootstrap, setBootstrap] = useState<DesktopBootstrap | null>(null);
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);
  const [sessionSnapshot, setSessionSnapshot] = useState<DesktopSessionSnapshot | null>(null);
  const [workspaceSnapshot, setWorkspaceSnapshot] = useState<DesktopWorkspaceSnapshot | null>(null);
  const [approvalState] = useState<ApprovalUiState>(() => createApprovalState());

  useEffect(() => {
    let mounted = true;
    desktopBootstrap()
      .then(async (nextBootstrap) => {
        const nextSession = await startSession({
          workspace_path: null,
          client_version: nextBootstrap.app_version,
        });
        const nextWorkspaceSnapshot = await loadWorkspaceSnapshot();
        if (mounted) {
          setBootstrap(nextBootstrap);
          setSessionSnapshot(nextSession);
          setWorkspaceSnapshot(nextWorkspaceSnapshot);
          setBootstrapError(null);
        }
      })
      .catch((error: unknown) => {
        if (mounted) {
          setBootstrapError(error instanceof Error ? error.message : String(error));
        }
      });

    return () => {
      mounted = false;
    };
  }, []);

  async function handleSubmitPrompt(prompt: string) {
    if (
      workspaceSnapshot?.active_conversation_id === null ||
      workspaceSnapshot?.active_conversation_id === undefined ||
      bootstrap === null
    ) {
      return;
    }

    try {
      await sendUiCommand(
        createStartTurnEnvelope({
          workspaceId: workspaceSnapshot.workspace_id,
          conversationId: workspaceSnapshot.active_conversation_id,
          prompt,
          clientVersion: bootstrap.app_version,
        }),
      );
      setBootstrapError(null);
    } catch (error) {
      setBootstrapError(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleAllowApproval(requestId: string) {
    await handleResolveApproval(requestId, 'allow', 'approved in desktop');
  }

  async function handleDenyApproval(requestId: string, reason: string) {
    await handleResolveApproval(requestId, 'deny', reason);
  }

  async function handleResolveApproval(
    requestId: string,
    decision: 'allow' | 'deny',
    reason: string,
  ) {
    if (bootstrap === null) {
      return;
    }

    try {
      await resolveApproval(
        {
          request_id: requestId,
          decision,
          reason,
          lease_duration_seconds: decision === 'allow' ? 120 : null,
        },
        bootstrap.app_version,
      );
      setBootstrapError(null);
    } catch (error) {
      setBootstrapError(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <AppShell
      bootstrap={bootstrap}
      bootstrapError={bootstrapError}
      sessionSnapshot={sessionSnapshot}
      workspaceSnapshot={workspaceSnapshot}
      approvalState={approvalState}
      onAllowApproval={(requestId) => {
        void handleAllowApproval(requestId);
      }}
      onDenyApproval={(requestId, reason) => {
        void handleDenyApproval(requestId, reason);
      }}
      onSubmitPrompt={(prompt) => {
        void handleSubmitPrompt(prompt);
      }}
    />
  );
}
