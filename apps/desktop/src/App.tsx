import { useEffect, useMemo, useState } from 'react';
import {
  Archive,
  Command,
  FolderOpen,
  PanelLeftClose,
  PanelRightClose,
  Pin,
  Plus,
  Search,
  Terminal,
  X,
} from 'lucide-react';

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
import { DiffPreview } from './components/DiffPreview';
import { ExtensionHealthView } from './components/ExtensionHealthView';
import { MemoryView } from './components/MemoryView';
import { SettingsView } from './components/SettingsView';
import { TaskDetail } from './components/TaskDetail';
import { TaskList } from './components/TaskList';
import { TeamWorkflowView } from './components/TeamWorkflowView';
import { TraceTimeline } from './components/TraceTimeline';
import { TranscriptView } from './components/TranscriptView';
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
  createSwitchSessionEnvelope,
  recordTurnSubmission,
  reduceBridgeError,
  reduceCommandAck,
  reduceConversationTaskFrames,
  selectConversation,
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
  DesktopDiffReview,
  DesktopInspectorTab,
  DesktopSettings,
  DesktopShellUiState,
  DesktopSessionSnapshot,
  DesktopWorkspaceSnapshot,
  ConversationTaskState,
  SettingsUiState,
  TraceTimeline as TraceTimelineModel,
  TraceUiState,
  UiExtensionSnapshot,
  UiMemorySnapshot,
  UiTeamWorkflowSnapshot,
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
  onSelectConversation = () => undefined,
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
  diffReview = null,
  teamWorkflowSnapshot = null,
  memorySnapshot = null,
}: {
  bootstrap: DesktopBootstrap | null;
  bootstrapError: string | null;
  dashboardSnapshot?: DesktopDashboardSnapshot | null;
  workspaceSnapshot?: DesktopWorkspaceSnapshot | null;
  conversationState?: ConversationTaskState | null;
  sessionSnapshot?: DesktopSessionSnapshot | null;
  onSubmitPrompt?: (prompt: string) => void;
  onSelectConversation?: (conversationId: string) => void;
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
  diffReview?: DesktopDiffReview | null;
  teamWorkflowSnapshot?: UiTeamWorkflowSnapshot | null;
  memorySnapshot?: UiMemorySnapshot | null;
}) {
  const [shellState, setShellState] = useState<DesktopShellUiState>({
    sidebarOpen: true,
    inspectorOpen: true,
    terminalOpen: false,
    commandPaletteOpen: false,
    findOpen: false,
    activeInspectorTab: 'context',
    lastCommand: null,
  });
  const [threadSearch, setThreadSearch] = useState('');
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
  const effectiveTeamWorkflowSnapshot =
    teamWorkflowSnapshot ?? conversationTaskState.teamWorkflowSnapshot;
  const effectiveMemorySnapshot = memorySnapshot ?? conversationTaskState.memorySnapshot;
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
  const filteredConversations = conversationTaskState.conversations.filter((conversation) =>
    conversation.title.toLowerCase().includes(threadSearch.trim().toLowerCase()),
  );
  const activeConversationTitle =
    conversationTaskState.conversations.find(
      (conversation) => conversation.conversation_id === conversationTaskState.activeConversationId,
    )?.title ?? 'Untitled thread';
  const selectedProvider =
    configSnapshot?.snapshot.selected_provider ?? settings.model.provider ?? 'provider';
  const selectedModel = configSnapshot?.snapshot.selected_model ?? settings.model.model ?? 'model';
  const activeProvider = configSnapshot?.snapshot.providers.find(
    (provider) => provider.id === selectedProvider,
  );

  function updateShellState(update: Partial<DesktopShellUiState>) {
    setShellState((state) => ({ ...state, ...update }));
  }

  function toggleShellFlag(flag: 'sidebarOpen' | 'inspectorOpen' | 'terminalOpen' | 'findOpen') {
    setShellState((state) => ({ ...state, [flag]: !state[flag] }));
  }

  function recordCommand(command: string) {
    setShellState((state) => ({
      ...state,
      commandPaletteOpen: false,
      lastCommand: command,
    }));
  }

  useEffect(() => {
    function handleShortcut(event: KeyboardEvent) {
      const target = event.target;
      const isTypingTarget =
        target instanceof HTMLElement &&
        (target.tagName === 'TEXTAREA' ||
          target.tagName === 'INPUT' ||
          target.isContentEditable);
      const isModified = event.metaKey || event.ctrlKey;
      if (!isModified) {
        return;
      }

      const key = event.key.toLowerCase();
      if (isTypingTarget && key !== 'k') {
        return;
      }

      if (key === 'k') {
        event.preventDefault();
        setShellState((state) => ({
          ...state,
          commandPaletteOpen: !state.commandPaletteOpen,
        }));
        return;
      }

      if (key === 'n') {
        event.preventDefault();
        recordCommand('New thread shortcut captured');
        return;
      }

      if (key === 'b') {
        event.preventDefault();
        toggleShellFlag('sidebarOpen');
        return;
      }

      if (key === 'i') {
        event.preventDefault();
        toggleShellFlag('inspectorOpen');
        return;
      }

      if (key === 'j') {
        event.preventDefault();
        toggleShellFlag('terminalOpen');
        return;
      }

      if (key === ',') {
        event.preventDefault();
        recordCommand('Settings shortcut captured');
        return;
      }

      if (key === 'f') {
        event.preventDefault();
        toggleShellFlag('findOpen');
      }
    }

    window.addEventListener('keydown', handleShortcut);
    return () => window.removeEventListener('keydown', handleShortcut);
  }, []);

  return (
    <div
      className={[
        'desktop-shell',
        shellState.sidebarOpen ? '' : 'sidebar-collapsed',
        shellState.inspectorOpen ? '' : 'inspector-collapsed',
        shellState.terminalOpen ? 'terminal-open' : '',
      ]
        .filter(Boolean)
        .join(' ')}
    >
      {shellState.commandPaletteOpen ? (
        <div className="command-overlay" role="dialog" aria-label="Command palette">
          <section className="command-palette">
            <header>
              <span>
                <Command aria-hidden="true" size={16} />
                Command Palette
              </span>
              <button
                aria-label="Close command palette"
                className="icon-button"
                type="button"
                onClick={() => updateShellState({ commandPaletteOpen: false })}
              >
                <X aria-hidden="true" size={16} />
              </button>
            </header>
            <input aria-label="Command search" autoFocus placeholder="Search commands" />
            <div className="command-list" role="list">
              <button type="button" onClick={() => recordCommand('New thread')}>
                <span>New thread</span>
                <kbd>Cmd N</kbd>
              </button>
              <button type="button" onClick={() => toggleShellFlag('sidebarOpen')}>
                <span>Toggle sidebar</span>
                <kbd>Cmd B</kbd>
              </button>
              <button type="button" onClick={() => toggleShellFlag('inspectorOpen')}>
                <span>Toggle inspector</span>
                <kbd>Cmd I</kbd>
              </button>
              <button type="button" onClick={() => toggleShellFlag('terminalOpen')}>
                <span>Toggle terminal</span>
                <kbd>Cmd J</kbd>
              </button>
              <button type="button" onClick={() => toggleShellFlag('findOpen')}>
                <span>Find in thread</span>
                <kbd>Cmd F</kbd>
              </button>
              <button type="button" onClick={() => recordCommand('Settings')}>
                <span>Settings</span>
                <kbd>Cmd ,</kbd>
              </button>
            </div>
          </section>
        </div>
      ) : null}

      <nav className="left-rail" aria-label="Workspace and thread navigation">
        <div className="brand" aria-label="Application identity">
          <span className="brand-mark">A</span>
          <div>
            <span className="brand-name">{bootstrap?.app_name ?? 'agent'}</span>
            <span className="brand-version">{bootstrap?.app_version ?? 'starting'}</span>
          </div>
        </div>
        <button className="workspace-switcher" type="button">
          <FolderOpen aria-hidden="true" size={16} />
          <span>
            <small>Workspace</small>
            <strong>{dashboardSnapshot?.workspace_root_display ?? 'Local workspace'}</strong>
          </span>
        </button>
        <button
          aria-label="+ New Thread"
          className="primary-action"
          type="button"
          onClick={() => recordCommand('New thread')}
        >
          <Plus aria-hidden="true" size={16} />
          New Thread
        </button>
        <label className="thread-search-shell">
          <Search aria-hidden="true" size={15} />
          <input
            aria-label="Search threads"
            className="thread-search"
            placeholder="Search threads"
            value={threadSearch}
            onChange={(event) => setThreadSearch(event.currentTarget.value)}
          />
        </label>
        <ThreadSection
          activeConversationId={conversationTaskState.activeConversationId}
          conversations={filteredConversations}
          emptyLabel="No recent threads"
          label="Recent"
          onSelect={onSelectConversation}
        />
        <ThreadSection
          activeConversationId={conversationTaskState.activeConversationId}
          conversations={[]}
          emptyLabel="No pinned threads"
          label="Pinned"
          onSelect={() => undefined}
        />
        <ThreadSection
          activeConversationId={conversationTaskState.activeConversationId}
          conversations={[]}
          emptyLabel="No archived threads"
          label="Archived"
          onSelect={() => undefined}
        />
      </nav>

      <main className="workspace" aria-label="Active thread workspace">
        <header className="workspace-toolbar">
          <div>
            <h1>{activeConversationTitle}</h1>
            <p>
              {conversationTaskState.activeConversationId ?? 'No active thread'} ·{' '}
              {sessionSnapshot?.state ?? bootstrap?.runtime_state ?? 'loading'}
            </p>
          </div>
          <div className="toolbar-actions" aria-label="Workspace controls">
            <button
              aria-label="Toggle sidebar"
              className="icon-button"
              type="button"
              onClick={() => toggleShellFlag('sidebarOpen')}
            >
              <PanelLeftClose aria-hidden="true" size={17} />
            </button>
            <button
              aria-label="Open command palette"
              className="icon-button"
              type="button"
              onClick={() => updateShellState({ commandPaletteOpen: true })}
            >
              <Command aria-hidden="true" size={17} />
            </button>
            <button
              aria-label="Toggle inspector"
              className="icon-button"
              type="button"
              onClick={() => toggleShellFlag('inspectorOpen')}
            >
              <PanelRightClose aria-hidden="true" size={17} />
            </button>
            <button
              aria-label="Toggle terminal"
              className="icon-button"
              type="button"
              onClick={() => toggleShellFlag('terminalOpen')}
            >
              <Terminal aria-hidden="true" size={17} />
            </button>
          </div>
        </header>
        <section className="thread-control-strip" aria-label="Thread controls">
          <div>
            <span className="status-dot" aria-hidden="true" />
            <strong>{selectedProvider}</strong>
            <span>{selectedModel}</span>
          </div>
          <div>
            <span>{activeProvider?.kind ?? 'configured'}</span>
            <span>{activeProvider?.capabilities?.max_context_tokens ?? 'context'} tokens</span>
          </div>
        </section>
        {shellState.findOpen ? (
          <div className="find-bar">
            <input aria-label="Find in thread" placeholder="Find in thread" />
            <button type="button" onClick={() => updateShellState({ findOpen: false })}>
              <X aria-hidden="true" size={15} />
              Close
            </button>
          </div>
        ) : null}
        {shellState.lastCommand === null ? null : (
          <div className="command-notice" role="status">
            {shellState.lastCommand}
          </div>
        )}
        <section className="transcript-band">
          <TranscriptView entries={conversationTaskState.transcript} />
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
        {shellState.terminalOpen ? (
          <section className="terminal-drawer" aria-label="Integrated terminal">
            <header>
              <span>Terminal</span>
              <button type="button" onClick={() => toggleShellFlag('terminalOpen')}>
                <X aria-hidden="true" size={15} />
                Hide
              </button>
            </header>
            <p>No terminal session</p>
          </section>
        ) : null}
      </main>

      <aside className="context-panel" aria-label="Inspector">
        <header className="inspector-header">
          <h2>Inspector</h2>
          <span>{approvalState.pending.length} pending</span>
          <button
            aria-label="Collapse inspector"
            className="icon-button"
            type="button"
            onClick={() => toggleShellFlag('inspectorOpen')}
          >
            <X aria-hidden="true" size={16} />
          </button>
        </header>
        <div className="inspector-tabs" role="tablist" aria-label="Inspector tabs">
          {inspectorTabs.map((tab) => (
            <button
              aria-controls={`inspector-${tab}`}
              aria-selected={shellState.activeInspectorTab === tab}
              className={shellState.activeInspectorTab === tab ? 'active' : ''}
              id={`tab-${tab}`}
              key={tab}
              role="tab"
              type="button"
              onClick={() => updateShellState({ activeInspectorTab: tab })}
            >
              {inspectorTabLabel(tab)}
            </button>
          ))}
        </div>
        <section
          aria-labelledby={`tab-${shellState.activeInspectorTab}`}
          className="inspector-content"
          id={`inspector-${shellState.activeInspectorTab}`}
          role="tabpanel"
        >
          {shellState.activeInspectorTab === 'context' ? (
            <>
              {bootstrapError ? <BootstrapError message={bootstrapError} /> : null}
              <section className="panel-section">
                <h3>Runtime</h3>
                <dl>
                  <dt>State</dt>
                  <dd>{bootstrap?.runtime_state ?? 'loading'}</dd>
                  <dt>Session</dt>
                  <dd>{sessionSnapshot?.session_id ?? 'not connected'}</dd>
                  <dt>Frames</dt>
                  <dd>{sessionSnapshot?.pending_frame_count ?? 0}</dd>
                </dl>
              </section>
              <section className="panel-section">
                <h3>Diagnostics</h3>
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
              <SettingsView
                snapshot={configSnapshot}
                settings={settings}
                diagnostics={settingsDiagnostics}
                error={settingsError}
              />
              {extensionSnapshot === null ? null : (
                <ExtensionHealthView snapshot={extensionSnapshot} />
              )}
            </>
          ) : null}
          {shellState.activeInspectorTab === 'changes' ? (
            selectedApproval?.preview === undefined ? (
              <div className="empty-list">No changes to review</div>
            ) : (
              <DiffPreview preview={selectedApproval.preview} review={diffReview} />
            )
          ) : null}
          {shellState.activeInspectorTab === 'approvals' ? (
            <>
              <ApprovalQueue
                approvals={approvalState.pending}
                selectedRequestId={approvalState.selectedRequestId}
                onSelect={onSelectApproval}
              />
              <ApprovalDetail
                approval={selectedApproval}
                decisionAck={approvalState.lastDecisionAck}
                decisionError={approvalState.lastDecisionError}
                diffReview={diffReview}
                onAllow={onAllowApproval}
                onDeny={onDenyApproval}
              />
            </>
          ) : null}
          {shellState.activeInspectorTab === 'tasks' ? (
            <>
              {effectiveMemorySnapshot === null ? null : (
                <MemoryView snapshot={effectiveMemorySnapshot} />
              )}
              {effectiveTeamWorkflowSnapshot === null ? null : (
                <TeamWorkflowView snapshot={effectiveTeamWorkflowSnapshot} />
              )}
              <TaskList
                activeTaskId={conversationTaskState.activeTaskId}
                tasks={conversationTaskState.tasks}
                onSelect={() => undefined}
              />
              <TaskDetail task={selectedTask} />
            </>
          ) : null}
          {shellState.activeInspectorTab === 'artifacts' ? (
            <div className="empty-list">No artifacts yet</div>
          ) : null}
          {shellState.activeInspectorTab === 'trace' ? (
            <TraceTimeline timeline={traceTimeline} error={traceError} />
          ) : null}
        </section>
      </aside>

      <footer className="status-bar">
        <span>{bootstrap?.app_name ?? 'agent'}</span>
        <span>{bootstrap?.app_version ?? '0.1.0'}</span>
        <span>{bootstrap?.runtime_state ?? 'loading'}</span>
        <span>{selectedProvider}</span>
      </footer>
    </div>
  );
}

function ThreadSection({
  activeConversationId,
  conversations,
  emptyLabel,
  label,
  onSelect,
}: {
  activeConversationId: string | null;
  conversations: ConversationTaskState['conversations'];
  emptyLabel: string;
  label: string;
  onSelect: (conversationId: string) => void;
}) {
  return (
    <section className="thread-section" aria-label={`${label} threads`}>
      <h2>{label}</h2>
      {label === 'Pinned' ? <Pin aria-hidden="true" className="thread-section-icon" size={13} /> : null}
      {label === 'Archived' ? (
        <Archive aria-hidden="true" className="thread-section-icon" size={13} />
      ) : null}
      {conversations.length === 0 ? (
        <div className="empty-list">{emptyLabel}</div>
      ) : (
        <ConversationList
          activeConversationId={activeConversationId}
          conversations={conversations}
          onSelect={onSelect}
        />
      )}
    </section>
  );
}

const inspectorTabs: DesktopInspectorTab[] = [
  'context',
  'changes',
  'approvals',
  'tasks',
  'artifacts',
  'trace',
];

function inspectorTabLabel(tab: DesktopInspectorTab): string {
  switch (tab) {
    case 'context':
      return 'Context';
    case 'changes':
      return 'Changes';
    case 'approvals':
      return 'Approvals';
    case 'tasks':
      return 'Tasks';
    case 'artifacts':
      return 'Artifacts';
    case 'trace':
      return 'Trace';
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

  async function handleSelectConversation(conversationId: string) {
    if (workspaceSnapshot === null || bootstrap === null) {
      return;
    }

    setConversationState((state) => selectConversation(state, conversationId));
    setWorkspaceSnapshot((snapshot) =>
      snapshot === null
        ? snapshot
        : {
            ...snapshot,
            active_conversation_id: conversationId,
          },
    );

    try {
      const envelope = createSwitchSessionEnvelope({
        workspaceId: workspaceSnapshot.workspace_id,
        conversationId,
        clientVersion: bootstrap.app_version,
      });
      const ack = await sendUiCommand(envelope);
      setConversationState((state) => reduceCommandAck(state, ack));
      await drainBridgeFrames(32);
      const timeline = await loadTrace(conversationId);
      setTraceState(
        reduceTraceLoaded(createTraceState(conversationId), timeline),
      );
      setBootstrapError(null);
    } catch (error) {
      const normalized = normalizeBridgeError(error);
      setConversationState((state) => reduceBridgeError(state, normalized));
      setBootstrapError(normalized.message);
    }
  }

  async function handleSubmitPrompt(prompt: string) {
    const activeConversationId = conversationState.activeConversationId;
    if (workspaceSnapshot === null || activeConversationId === null || bootstrap === null) {
      return;
    }

    try {
      const envelope = createStartTurnEnvelope({
        workspaceId: workspaceSnapshot.workspace_id,
        conversationId: activeConversationId,
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
      onAllowApproval={(requestId) => {
        void handleAllowApproval(requestId);
      }}
      onDenyApproval={(requestId, reason) => {
        void handleDenyApproval(requestId, reason);
      }}
      onSubmitPrompt={(prompt) => {
        void handleSubmitPrompt(prompt);
      }}
      onSelectConversation={(conversationId) => {
        void handleSelectConversation(conversationId);
      }}
      configSnapshot={settingsState.snapshot}
      settings={settingsState.settings}
      settingsDiagnostics={settingsState.validationDiagnostics}
      settingsError={settingsState.error}
    />
  );
}
