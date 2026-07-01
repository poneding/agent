import { useEffect, useMemo, useState } from 'react';

import {
  desktopBootstrap,
  loadDashboardSnapshot,
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
import { ExtensionHealthView } from './components/ExtensionHealthView';
import { SettingsView } from './components/SettingsView';
import { TaskDetail } from './components/TaskDetail';
import { TaskList } from './components/TaskList';
import { TraceTimeline } from './components/TraceTimeline';
import { TranscriptView } from './components/TranscriptView';
import { WorkspaceDashboard } from './components/WorkspaceDashboard';
import { drainUiFrames, normalizeBridgeError, sendUiCommand, startSession } from './bridge';
import {
  createApprovalState,
  reduceApprovalCommandAck,
  reduceApprovalError,
  reduceApprovalState,
  selectApproval,
} from './state/approvals';
import {
  createConversationTaskState,
  createStartTurnEnvelope,
  recordTurnSubmission,
  reduceBridgeError,
  reduceCommandAck,
  reduceConversationTaskFrames,
} from './state/conversation';
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
  DesktopDashboardSnapshot,
  DesktopSettings,
  DesktopShellRoute,
  DesktopSessionSnapshot,
  DesktopWorkspaceSnapshot,
  ConversationTaskState,
  SettingsUiState,
  TraceTimeline as TraceTimelineModel,
  TraceUiState,
  UiExtensionSnapshot,
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
  dashboardSnapshot = null,
  workspaceSnapshot,
  conversationState = null,
  sessionSnapshot,
  onSubmitPrompt = () => undefined,
  approvalState = createApprovalState(),
  onSelectApproval = () => undefined,
  onAllowApproval = () => undefined,
  onDenyApproval = () => undefined,
  traceTimeline = null,
  traceError = null,
  configSnapshot = null,
  settings = defaultDesktopSettings(),
  settingsDiagnostics = [],
  settingsError = null,
  extensionSnapshot = null,
}: {
  bootstrap: DesktopBootstrap | null;
  bootstrapError: string | null;
  dashboardSnapshot?: DesktopDashboardSnapshot | null;
  workspaceSnapshot?: DesktopWorkspaceSnapshot | null;
  conversationState?: ConversationTaskState | null;
  sessionSnapshot?: DesktopSessionSnapshot | null;
  onSubmitPrompt?: (prompt: string) => void;
  approvalState?: ApprovalUiState;
  onSelectApproval?: (requestId: string) => void;
  onAllowApproval?: (requestId: string) => void;
  onDenyApproval?: (requestId: string, reason: string) => void;
  traceTimeline?: TraceTimelineModel | null;
  traceError?: string | null;
  configSnapshot?: DesktopConfigSnapshot | null;
  settings?: DesktopSettings;
  settingsDiagnostics?: DesktopBootstrapDiagnostic[];
  settingsError?: string | null;
  extensionSnapshot?: UiExtensionSnapshot | null;
}) {
  const [route, setRoute] = useState<DesktopShellRoute>(bootstrap?.route ?? 'dashboard');
  const diagnostics = bootstrap?.diagnostics ?? [];
  const fallbackConversationTaskState = useMemo(
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
  const conversationTaskState = conversationState ?? fallbackConversationTaskState;
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
          className={route === 'dashboard' ? 'rail-item active' : 'rail-item'}
          type="button"
          onClick={() => setRoute('dashboard')}
        >
          Dashboard
        </button>
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
        {route === 'dashboard' ? (
          <WorkspaceDashboard
            snapshot={dashboardSnapshot}
            onOpenConversation={() => setRoute('conversation')}
            onOpenApprovals={() => setRoute('approvals')}
          />
        ) : null}
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
              {conversationTaskState.diagnostics.length === 0 ? null : (
                <ul className="conversation-diagnostics" aria-label="Conversation diagnostics">
                  {conversationTaskState.diagnostics.map((diagnostic) => (
                    <li key={`${diagnostic.code}-${diagnostic.message}`}>
                      <strong>{diagnostic.code}</strong>
                      <span>{diagnostic.message}</span>
                    </li>
                  ))}
                </ul>
              )}
            </section>
            <Composer
              disabled={composerDisabled}
              onSubmitPrompt={onSubmitPrompt}
              submission={conversationTaskState.currentSubmission}
            />
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
                  onSelect={onSelectApproval}
                />
              </div>
              <ApprovalDetail
                approval={selectedApproval}
                decisionAck={approvalState.lastDecisionAck}
                decisionError={approvalState.lastDecisionError}
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
            {extensionSnapshot === null ? null : (
              <ExtensionHealthView snapshot={extensionSnapshot} />
            )}
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
            onSelect={onSelectApproval}
          />
          <ApprovalDetail
            approval={selectedApproval}
            decisionAck={approvalState.lastDecisionAck}
            decisionError={approvalState.lastDecisionError}
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
    case 'dashboard':
      return 'Dashboard';
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
  const [dashboardSnapshot, setDashboardSnapshot] = useState<DesktopDashboardSnapshot | null>(null);
  const [sessionSnapshot, setSessionSnapshot] = useState<DesktopSessionSnapshot | null>(null);
  const [workspaceSnapshot, setWorkspaceSnapshot] = useState<DesktopWorkspaceSnapshot | null>(null);
  const [conversationState, setConversationState] = useState<ConversationTaskState>(() =>
    createConversationTaskState({
      workspace_id: 'loading',
      active_conversation_id: null,
      conversations: [],
      tasks: [],
    }),
  );
  const [approvalState, setApprovalState] = useState<ApprovalUiState>(() => createApprovalState());
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
        const nextDashboardSnapshot = await loadDashboardSnapshot();
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
          setDashboardSnapshot(nextDashboardSnapshot);
          setSessionSnapshot(nextSession);
          setWorkspaceSnapshot(nextWorkspaceSnapshot);
          setConversationState(createConversationTaskState(nextWorkspaceSnapshot));
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
      const envelope = createStartTurnEnvelope({
        workspaceId: workspaceSnapshot.workspace_id,
        conversationId: workspaceSnapshot.active_conversation_id,
        prompt,
        clientVersion: bootstrap.app_version,
      });
      setConversationState((state) => recordTurnSubmission(state, envelope));
      const ack = await sendUiCommand(envelope);
      setConversationState((state) => reduceCommandAck(state, ack));
      setSessionSnapshot((snapshot) =>
        snapshot === null
          ? snapshot
          : {
              ...snapshot,
              state: ack.state,
              pending_frame_count: ack.pending_frame_count,
            },
      );
      await drainBridgeFrames(32);
      setBootstrapError(null);
    } catch (error) {
      const normalized = normalizeBridgeError(error);
      setConversationState((state) => reduceBridgeError(state, normalized));
      setBootstrapError(normalized.message);
    }
  }

  async function drainBridgeFrames(maxCount: number) {
    try {
      const frames = await drainUiFrames(maxCount);
      if (frames.length > 0) {
        setConversationState((state) => reduceConversationTaskFrames(state, frames));
        setApprovalState((state) =>
          frames.reduce((nextState, frame) => reduceApprovalState(nextState, frame), state),
        );
      }
    } catch (error) {
      const normalized = normalizeBridgeError(error);
      setConversationState((state) => reduceBridgeError(state, normalized));
      setBootstrapError(normalized.message);
    }
  }

  useEffect(() => {
    if (sessionSnapshot?.state !== 'connected') {
      return undefined;
    }

    const interval = window.setInterval(() => {
      void drainBridgeFrames(32);
    }, 1000);

    return () => window.clearInterval(interval);
  }, [sessionSnapshot?.state]);

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
      const ack = await resolveApproval(
        {
          request_id: requestId,
          decision,
          reason,
          lease_duration_seconds: decision === 'allow' ? 120 : null,
        },
        bootstrap.app_version,
      );
      setApprovalState((state) => reduceApprovalCommandAck(state, ack));
      setBootstrapError(null);
    } catch (error) {
      const normalized = normalizeBridgeError(error);
      setApprovalState((state) => reduceApprovalError(state, normalized));
      setBootstrapError(normalized.message);
    }
  }

  return (
    <AppShell
      bootstrap={bootstrap}
      bootstrapError={bootstrapError}
      dashboardSnapshot={dashboardSnapshot}
      sessionSnapshot={sessionSnapshot}
      workspaceSnapshot={workspaceSnapshot}
      conversationState={conversationState}
      approvalState={approvalState}
      onSelectApproval={(requestId) => {
        setApprovalState((state) => selectApproval(state, requestId));
      }}
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
