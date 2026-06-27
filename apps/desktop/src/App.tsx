import { useEffect, useMemo, useState } from 'react';

import {
  desktopBootstrap,
  loadConfigSnapshot,
  loadTrace,
  loadWorkspaceSnapshot,
  resolveApproval,
  validateSettings,
} from './api';
import { ApprovalDetail } from './components/ApprovalDetail';
import { ApprovalQueue } from './components/ApprovalQueue';
import { Composer } from './components/Composer';
import { ConversationList } from './components/ConversationList';
import { SettingsView } from './components/SettingsView';
import { TaskDetail } from './components/TaskDetail';
import { TaskList } from './components/TaskList';
import { TraceTimeline } from './components/TraceTimeline';
import { TranscriptView } from './components/TranscriptView';
import { sendUiCommand, startSession } from './bridge';
import {
  createConversationTaskState,
  createStartTurnEnvelope,
} from './state/conversation';
import { createApprovalState } from './state/approvals';
import {
  createSettingsState,
  defaultDesktopSettings,
  reduceSettingsSnapshotLoaded,
  reduceSettingsValidation,
} from './state/settings';
import { selectTask } from './state/tasks';
import { createTraceState, reduceTraceLoaded } from './state/trace';
import type {
  ApprovalUiState,
  DesktopBootstrap,
  DesktopBootstrapDiagnostic,
  DesktopConfigSnapshot,
  DesktopSettings,
  DesktopShellRoute,
  DesktopSessionSnapshot,
  DesktopWorkspaceSnapshot,
  SettingsUiState,
  TraceTimeline as TraceTimelineModel,
  TraceUiState,
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
  traceTimeline = null,
  traceError = null,
  configSnapshot = null,
  settings = defaultDesktopSettings(),
  settingsDiagnostics = [],
  settingsError = null,
}: {
  bootstrap: DesktopBootstrap | null;
  bootstrapError: string | null;
  workspaceSnapshot?: DesktopWorkspaceSnapshot | null;
  sessionSnapshot?: DesktopSessionSnapshot | null;
  onSubmitPrompt?: (prompt: string) => void;
  approvalState?: ApprovalUiState;
  onAllowApproval?: (requestId: string) => void;
  onDenyApproval?: (requestId: string, reason: string) => void;
  traceTimeline?: TraceTimelineModel | null;
  traceError?: string | null;
  configSnapshot?: DesktopConfigSnapshot | null;
  settings?: DesktopSettings;
  settingsDiagnostics?: DesktopBootstrapDiagnostic[];
  settingsError?: string | null;
}) {
  const [route, setRoute] = useState<DesktopShellRoute>(bootstrap?.route ?? 'conversation');
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
  const routeLabel = routeLabelFor(route);

  return (
    <div className="desktop-shell">
      <nav className="left-rail" aria-label="Desktop navigation">
        <div className="brand">
          <span className="brand-name">{bootstrap?.app_name ?? 'agent'}</span>
          <span className="brand-version">{bootstrap?.app_version ?? 'starting'}</span>
        </div>
        <button
          className={route === 'conversation' ? 'rail-item active' : 'rail-item'}
          type="button"
          onClick={() => setRoute('conversation')}
        >
          Conversations
        </button>
        <button
          className={route === 'tasks' ? 'rail-item active' : 'rail-item'}
          type="button"
          onClick={() => setRoute('tasks')}
        >
          Tasks
        </button>
        <button
          className={route === 'approvals' ? 'rail-item active' : 'rail-item'}
          type="button"
          onClick={() => setRoute('approvals')}
        >
          Approvals
        </button>
        <button
          className={route === 'trace' ? 'rail-item active' : 'rail-item'}
          type="button"
          onClick={() => setRoute('trace')}
        >
          Trace
        </button>
        <button
          className={route === 'settings' ? 'rail-item active' : 'rail-item'}
          type="button"
          onClick={() => setRoute('settings')}
        >
          Settings
        </button>
      </nav>

      <main className="workspace" aria-label={`${routeLabel} workspace`}>
        {route === 'conversation' ? (
          <>
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
          </>
        ) : null}
        {route === 'tasks' ? (
          <section className="workspace-band">
            <h1>Tasks</h1>
            <div className="split-layout">
              <TaskList
                activeTaskId={conversationTaskState.activeTaskId}
                tasks={conversationTaskState.tasks}
                onSelect={() => undefined}
              />
              <TaskDetail task={selectedTask} />
            </div>
          </section>
        ) : null}
        {route === 'approvals' ? (
          <section className="workspace-band">
            <h1>Approvals</h1>
            <div className="split-layout">
              <div aria-label="Pending approvals">
                <ApprovalQueue
                  approvals={approvalState.pending}
                  selectedRequestId={approvalState.selectedRequestId}
                  onSelect={() => undefined}
                />
              </div>
              <ApprovalDetail
                approval={selectedApproval}
                onAllow={onAllowApproval}
                onDeny={onDenyApproval}
              />
            </div>
          </section>
        ) : null}
        {route === 'trace' ? (
          <section className="workspace-band">
            <h1>Trace</h1>
            <TraceTimeline timeline={traceTimeline} error={traceError} />
          </section>
        ) : null}
        {route === 'settings' ? (
          <section className="workspace-band">
            <h1>Settings</h1>
            <SettingsView
              snapshot={configSnapshot}
              settings={settings}
              diagnostics={settingsDiagnostics}
              error={settingsError}
            />
          </section>
        ) : null}
      </main>

      <aside className="context-panel" aria-label="Context panel">
        {bootstrapError ? <BootstrapError message={bootstrapError} /> : null}
        <section className="panel-section">
          <h2>Runtime</h2>
          <dl>
            <dt>State</dt>
            <dd>{bootstrap?.runtime_state ?? 'loading'}</dd>
            <dt>Route</dt>
            <dd>{route}</dd>
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

function routeLabelFor(route: DesktopShellRoute): string {
  switch (route) {
    case 'conversation':
      return 'Conversation';
    case 'tasks':
      return 'Tasks';
    case 'approvals':
      return 'Approvals';
    case 'trace':
      return 'Trace';
    case 'settings':
      return 'Settings';
  }
}

export default function App() {
  const [bootstrap, setBootstrap] = useState<DesktopBootstrap | null>(null);
  const [bootstrapError, setBootstrapError] = useState<string | null>(null);
  const [sessionSnapshot, setSessionSnapshot] = useState<DesktopSessionSnapshot | null>(null);
  const [workspaceSnapshot, setWorkspaceSnapshot] = useState<DesktopWorkspaceSnapshot | null>(null);
  const [approvalState] = useState<ApprovalUiState>(() => createApprovalState());
  const [traceState, setTraceState] = useState<TraceUiState>(() => createTraceState(null));
  const [settingsState, setSettingsState] = useState<SettingsUiState>(() => createSettingsState());

  useEffect(() => {
    let mounted = true;
    desktopBootstrap()
      .then(async (nextBootstrap) => {
        const nextSession = await startSession({
          workspace_path: null,
          client_version: nextBootstrap.app_version,
        });
        const nextWorkspaceSnapshot = await loadWorkspaceSnapshot();
        const nextConfigSnapshot = await loadConfigSnapshot();
        const nextSettingsState = reduceSettingsSnapshotLoaded(
          createSettingsState(),
          nextConfigSnapshot,
        );
        const nextSettingsDiagnostics = await validateSettings(nextSettingsState.settings);
        const nextTraceTimeline =
          nextWorkspaceSnapshot.active_conversation_id === null ||
          nextWorkspaceSnapshot.active_conversation_id === undefined
            ? null
            : await loadTrace(nextWorkspaceSnapshot.active_conversation_id);
        if (mounted) {
          setBootstrap(nextBootstrap);
          setSessionSnapshot(nextSession);
          setWorkspaceSnapshot(nextWorkspaceSnapshot);
          setSettingsState(
            reduceSettingsValidation(nextSettingsState, nextSettingsDiagnostics),
          );
          setTraceState(
            nextTraceTimeline === null
              ? createTraceState(null)
              : reduceTraceLoaded(
                  createTraceState(nextTraceTimeline.conversation_id),
                  nextTraceTimeline,
                ),
          );
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
      traceTimeline={traceState.timeline}
      traceError={traceState.error}
      configSnapshot={settingsState.snapshot}
      settings={settingsState.settings}
      settingsDiagnostics={settingsState.validationDiagnostics}
      settingsError={settingsState.error}
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
