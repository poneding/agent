import type {
  BridgeErrorNotice,
  ConversationDiagnostic,
  ConversationTaskState,
  DesktopBridgeError,
  DesktopCommandAck,
  DesktopTaskActiveTodo,
  DesktopTaskItem,
  DesktopTaskState,
  DesktopWorkspaceSnapshot,
  TranscriptEntry,
  TurnCommandAck,
  TurnSubmission,
  UiCommandEnvelope,
  UiJsonFrame,
  UiMemorySnapshot,
  UiPlanTodoItem,
  UiPlanTodoSnapshot,
  UiTeamWorkflowSnapshot,
} from '../types';

const MAX_CONVERSATION_TEXT_CHARS = 240;

interface StartTurnInput {
  workspaceId: string;
  conversationId: string;
  prompt: string;
  clientVersion: string;
}

interface SwitchConversationInput {
  workspaceId: string;
  conversationId: string;
  clientVersion: string;
}

export function createConversationTaskState(
  snapshot: DesktopWorkspaceSnapshot,
): ConversationTaskState {
  return {
    workspaceId: snapshot.workspace_id,
    conversations: snapshot.conversations,
    tasks: snapshot.tasks,
    transcript: [],
    transcriptsByConversation: {},
    activeConversationId: snapshot.active_conversation_id ?? null,
    activeTaskId: snapshot.tasks[0]?.task_id ?? null,
    composerText: '',
    currentSubmission: null,
    diagnostics: [],
    pendingFrameCount: 0,
    teamWorkflowSnapshot: null,
    memorySnapshot: null,
  };
}

export function reduceConversationTaskState(
  state: ConversationTaskState,
  frame: UiJsonFrame,
): ConversationTaskState {
  if (frame.type === 'done' && isRecord(frame.data)) {
    const eventCount =
      typeof frame.data.event_count === 'number' ? frame.data.event_count : 0;
    return markSubmissionCompleted(state, eventCount);
  }

  if (frame.type === 'error' && isRecord(frame.data)) {
    return reduceBridgeError(state, bridgeErrorFromFrame(frame.data));
  }

  if (frame.type === 'plan_todo_snapshot' && isPlanTodoSnapshot(frame.data)) {
    return reducePlanTodoSnapshot(state, frame.data);
  }

  if (frame.type === 'team_workflow_snapshot' && isTeamWorkflowSnapshot(frame.data)) {
    return {
      ...state,
      teamWorkflowSnapshot: frame.data,
    };
  }

  if (frame.type === 'memory_snapshot' && isMemorySnapshot(frame.data)) {
    return {
      ...state,
      memorySnapshot: frame.data,
    };
  }

  if (frame.type !== 'event' || !isRecord(frame.data)) {
    return state;
  }

  const payload = frame.data.payload;
  if (!isRecord(payload) || typeof payload.type !== 'string') {
    return state;
  }

  if (payload.type === 'task_notification' && isRecord(payload.data)) {
    return reduceTaskNotification(state, frame.data, payload.data);
  }

  const conversationId =
    typeof frame.data.conversation_id === 'string'
      ? frame.data.conversation_id
      : state.activeConversationId;
  const conversations =
    payload.type === 'user_message' && isRecord(payload.data) && typeof payload.data.message === 'string'
      ? upsertConversationFromUserMessage(
          state.conversations,
          conversationId,
          payload.data.message,
          typeof frame.data.occurred_at === 'string'
            ? frame.data.occurred_at
            : new Date().toISOString(),
        )
      : state.conversations;
  if (conversationId === null) {
    return conversations === state.conversations ? state : { ...state, conversations };
  }

  const conversationTranscript = state.transcriptsByConversation[conversationId] ?? [];
  const transcriptEntry = transcriptEntryFromPayload(
    payload.type,
    payload.data,
    conversationTranscript,
  );
  if (transcriptEntry === null) {
    return conversations === state.conversations ? state : { ...state, conversations };
  }

  const nextTranscript = [...conversationTranscript, transcriptEntry];
  return {
    ...state,
    conversations,
    transcriptsByConversation: {
      ...state.transcriptsByConversation,
      [conversationId]: nextTranscript,
    },
    transcript:
      conversationId === state.activeConversationId ? nextTranscript : state.transcript,
  };
}

export function reduceConversationTaskFrames(
  state: ConversationTaskState,
  frames: UiJsonFrame[],
): ConversationTaskState {
  return frames.reduce(reduceConversationTaskState, state);
}

export function createStartTurnEnvelope(input: StartTurnInput): UiCommandEnvelope {
  return {
    schema_version: 1,
    command_id: createCommandId(),
    client: {
      kind: 'desktop',
      version: input.clientVersion,
    },
    command: {
      type: 'start_turn',
      data: {
        workspace_id: input.workspaceId,
        conversation_id: input.conversationId,
        prompt: input.prompt,
      },
    },
  };
}

export function createSwitchSessionEnvelope(
  input: SwitchConversationInput,
): UiCommandEnvelope {
  return {
    schema_version: 1,
    command_id: createCommandId(),
    client: {
      kind: 'desktop',
      version: input.clientVersion,
    },
    command: {
      type: 'switch_session',
      data: {
        workspace_id: input.workspaceId,
        conversation_id: input.conversationId,
      },
    },
  };
}

export function selectConversation(
  state: ConversationTaskState,
  conversationId: string,
): ConversationTaskState {
  if (!state.conversations.some((conversation) => conversation.conversation_id === conversationId)) {
    return state;
  }
  return {
    ...state,
    activeConversationId: conversationId,
    transcript: state.transcriptsByConversation[conversationId] ?? [],
    currentSubmission: null,
    pendingFrameCount: 0,
  };
}

export function recordTurnSubmission(
  state: ConversationTaskState,
  envelope: UiCommandEnvelope,
): ConversationTaskState {
  if (envelope.command.type !== 'start_turn') {
    return state;
  }

  const { prompt, conversation_id: conversationId } = envelope.command.data;
  const preview = boundConversationText(prompt);
  const submission: TurnSubmission = {
    command_id: envelope.command_id,
    conversation_id: conversationId,
    prompt_preview: preview.text,
    status: 'submitting',
    ack: null,
    error: null,
    truncated: preview.truncated,
  };

  return {
    ...state,
    activeConversationId: conversationId,
    currentSubmission: submission,
  };
}

export function reduceCommandAck(
  state: ConversationTaskState,
  ack: DesktopCommandAck,
): ConversationTaskState {
  const turnAck: TurnCommandAck = {
    command_id: ack.command_id,
    accepted: ack.accepted,
    pending_frame_count: ack.pending_frame_count,
  };

  if (state.currentSubmission === null || state.currentSubmission.command_id !== ack.command_id) {
    return {
      ...state,
      pendingFrameCount: ack.pending_frame_count,
    };
  }

  return {
    ...state,
    pendingFrameCount: ack.pending_frame_count,
    currentSubmission: {
      ...state.currentSubmission,
      status: ack.accepted ? 'accepted' : 'rejected',
      ack: turnAck,
      error: ack.accepted ? null : state.currentSubmission.error,
    },
  };
}

export function reduceBridgeError(
  state: ConversationTaskState,
  error: DesktopBridgeError | BridgeErrorNotice,
): ConversationTaskState {
  const notice = bridgeErrorNotice(error);
  return {
    ...state,
    currentSubmission:
      state.currentSubmission === null
        ? null
        : {
            ...state.currentSubmission,
            status: 'rejected',
            error: notice,
          },
    diagnostics: [
      ...state.diagnostics,
      {
        severity: notice.recoverable ? 'warning' : 'error',
        code: notice.code,
        message: notice.message,
        truncated: notice.truncated,
      },
    ],
  };
}

function reduceTaskNotification(
  state: ConversationTaskState,
  envelope: Record<string, unknown>,
  data: Record<string, unknown>,
): ConversationTaskState {
  const taskId = typeof data.task_id === 'string' ? data.task_id : null;
  const summary = typeof data.summary === 'string' ? data.summary : null;
  if (taskId === null || summary === null) {
    return state;
  }

  const nextTask: DesktopTaskItem = {
    task_id: taskId,
    workspace_id: state.workspaceId,
    conversation_id: typeof envelope.conversation_id === 'string' ? envelope.conversation_id : null,
    state: taskStateFromKind(typeof data.kind === 'string' ? data.kind : ''),
    summary,
    updated_at:
      typeof envelope.occurred_at === 'string' ? envelope.occurred_at : new Date().toISOString(),
  };

  const existingIndex = state.tasks.findIndex((task) => task.task_id === taskId);
  const tasks =
    existingIndex >= 0
      ? state.tasks.map((task, index) => (index === existingIndex ? nextTask : task))
      : [nextTask, ...state.tasks];

  return {
    ...state,
    tasks,
    activeTaskId: state.activeTaskId ?? taskId,
  };
}

function reducePlanTodoSnapshot(
  state: ConversationTaskState,
  snapshot: UiPlanTodoSnapshot,
): ConversationTaskState {
  const taskId = snapshot.snapshot.task_id ?? taskIdFromPlanItems(snapshot.snapshot.items);
  if (taskId === null) {
    return state;
  }

  const activeTodo = activeTodoFromItems(snapshot.snapshot.items);
  const tasks = state.tasks.map((task) =>
    task.task_id === taskId
      ? {
          ...task,
          plan_progress: snapshot.progress,
          active_todo: activeTodo,
          artifact_count: Math.max(task.artifact_count ?? 0, activeTodo?.evidence_count ?? 0),
        }
      : task,
  );

  return {
    ...state,
    tasks,
    activeTaskId: state.activeTaskId ?? taskId,
  };
}

function transcriptEntryFromPayload(
  type: string,
  data: unknown,
  transcript: TranscriptEntry[],
): TranscriptEntry | null {
  if (!isRecord(data)) {
    return null;
  }

  if (type === 'user_message' && typeof data.message === 'string') {
    return transcriptEntry(transcript, 'user', data.message);
  }
  if (type === 'model_delta' && typeof data.delta === 'string') {
    return transcriptEntry(transcript, 'assistant', data.delta);
  }
  if (type === 'final_response' && typeof data.message === 'string') {
    return transcriptEntry(transcript, 'assistant', data.message);
  }
  if (type === 'diagnostic' && typeof data.message === 'string') {
    return transcriptEntry(transcript, 'diagnostic', data.message);
  }
  if (type === 'progress' && typeof data.message === 'string') {
    return transcriptEntry(transcript, 'progress', data.message);
  }

  return null;
}

function markSubmissionCompleted(
  state: ConversationTaskState,
  eventCount: number,
): ConversationTaskState {
  const completionEntry =
    eventCount > 0
      ? transcriptEntry(state.transcript, 'system', `${eventCount} frame events processed`)
      : null;

  const transcript =
    completionEntry === null ? state.transcript : [...state.transcript, completionEntry];
  const transcriptsByConversation =
    state.activeConversationId === null
      ? state.transcriptsByConversation
      : {
          ...state.transcriptsByConversation,
          [state.activeConversationId]: transcript,
        };

  return {
    ...state,
    pendingFrameCount: 0,
    currentSubmission:
      state.currentSubmission === null
        ? null
        : {
            ...state.currentSubmission,
            status: 'completed',
          },
    transcript,
    transcriptsByConversation,
  };
}

function upsertConversationFromUserMessage(
  conversations: ConversationTaskState['conversations'],
  conversationId: string | null,
  message: string,
  occurredAt: string,
): ConversationTaskState['conversations'] {
  if (conversationId === null) {
    return conversations;
  }
  const index = conversations.findIndex(
    (conversation) => conversation.conversation_id === conversationId,
  );
  const title = summarizeConversationTitle(message);
  if (index < 0) {
    return [
      {
        conversation_id: conversationId,
        title,
        last_activity_at: occurredAt,
        unread_count: 0,
      },
      ...conversations,
    ];
  }

  const current = conversations[index];
  const next = {
    ...current,
    title: isPlaceholderConversationTitle(current.title) ? title : current.title,
    last_activity_at: occurredAt,
  };
  return conversations.map((conversation, conversationIndex) =>
    conversationIndex === index ? next : conversation,
  );
}

function summarizeConversationTitle(message: string): string {
  const redacted = boundConversationText(message).text;
  const sentence = redacted.split(/[.!?。！？\n]/u)[0]?.trim() ?? '';
  if (sentence.length === 0) {
    return 'New session';
  }
  const chars = Array.from(sentence);
  return chars.length > 56 ? `${chars.slice(0, 56).join('')}...` : sentence;
}

function isPlaceholderConversationTitle(title: string): boolean {
  const normalized = title.trim().toLowerCase();
  return (
    normalized.length === 0 ||
    normalized === 'untitled thread' ||
    normalized === 'new session' ||
    normalized === 'repo session'
  );
}

function transcriptEntry(
  transcript: TranscriptEntry[],
  kind: TranscriptEntry['kind'],
  content: string,
): TranscriptEntry {
  const bounded = boundConversationText(content);
  return {
    id: `entry-${transcript.length + 1}`,
    kind,
    content: bounded.text,
    truncated: bounded.truncated,
  };
}

function taskStateFromKind(kind: string): DesktopTaskState {
  const normalized = kind.trim().toLowerCase();
  if (
    normalized === 'created' ||
    normalized === 'queued' ||
    normalized === 'running' ||
    normalized === 'waiting_approval' ||
    normalized === 'sleeping' ||
    normalized === 'paused' ||
    normalized === 'succeeded' ||
    normalized === 'failed' ||
    normalized === 'cancelled' ||
    normalized === 'expired'
  ) {
    return normalized;
  }
  return 'running';
}

function isPlanTodoSnapshot(value: unknown): value is UiPlanTodoSnapshot {
  if (!isRecord(value) || !isRecord(value.snapshot) || !isRecord(value.progress)) {
    return false;
  }
  return (
    typeof value.snapshot.conversation_id === 'string' &&
    typeof value.snapshot.revision === 'number' &&
    Array.isArray(value.snapshot.items) &&
    typeof value.progress.total === 'number' &&
    typeof value.progress.pending === 'number' &&
    typeof value.progress.in_progress === 'number' &&
    typeof value.progress.completed === 'number' &&
    typeof value.progress.cancelled === 'number'
  );
}

function isTeamWorkflowSnapshot(value: unknown): value is UiTeamWorkflowSnapshot {
  if (!isRecord(value) || !isRecord(value.snapshot)) {
    return false;
  }
  return (
    typeof value.operation === 'string' &&
    typeof value.dry_run === 'boolean' &&
    typeof value.snapshot.status === 'string' &&
    typeof value.snapshot.readiness === 'string'
  );
}

function isMemorySnapshot(value: unknown): value is UiMemorySnapshot {
  if (!isRecord(value) || !isRecord(value.retention)) {
    return false;
  }
  return (
    typeof value.scope_label === 'string' &&
    typeof value.enabled === 'boolean' &&
    typeof value.layer_count === 'number' &&
    typeof value.record_count === 'number' &&
    typeof value.proposal_count === 'number' &&
    typeof value.selected_count === 'number' &&
    typeof value.rejected_count === 'number' &&
    typeof value.tombstone_count === 'number' &&
    Array.isArray(value.records) &&
    Array.isArray(value.proposals) &&
    Array.isArray(value.diagnostics) &&
    typeof value.retention.memory_enabled === 'boolean'
  );
}

function taskIdFromPlanItems(items: UiPlanTodoItem[]): string | null {
  for (const item of items) {
    if (typeof item.task_id === 'string' && item.task_id.length > 0) {
      return item.task_id;
    }
  }
  return null;
}

function activeTodoFromItems(items: UiPlanTodoItem[]): DesktopTaskActiveTodo | null {
  const item = items.find((candidate) => candidate.status === 'in_progress');
  if (item === undefined) {
    return null;
  }
  return {
    item_id: item.item_id,
    content: boundConversationText(item.content).text,
    priority: item.priority,
    evidence_count: Array.isArray(item.evidence) ? item.evidence.length : 0,
  };
}

function createCommandId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `desktop-command-${Date.now()}`;
}

function bridgeErrorFromFrame(data: Record<string, unknown>): BridgeErrorNotice {
  return bridgeErrorNotice({
    code: typeof data.code === 'string' ? data.code : 'desktop.bridge.error',
    message: typeof data.message === 'string' ? data.message : 'desktop bridge error',
    recoverable: typeof data.recoverable === 'boolean' ? data.recoverable : true,
  });
}

function bridgeErrorNotice(error: DesktopBridgeError | BridgeErrorNotice): BridgeErrorNotice {
  const code = boundConversationText(error.code);
  const message = boundConversationText(error.message);
  return {
    code: code.text || 'desktop.bridge.error',
    message: message.text || 'desktop bridge error',
    recoverable: error.recoverable,
    truncated: code.truncated || message.truncated || ('truncated' in error && error.truncated),
  };
}

function boundConversationText(value: string): { text: string; truncated: boolean } {
  const redacted = redactSecretLikeText(value).trim();
  if (redacted.length === 0) {
    return { text: '', truncated: false };
  }
  const chars = Array.from(redacted);
  if (chars.length > MAX_CONVERSATION_TEXT_CHARS) {
    return { text: chars.slice(0, MAX_CONVERSATION_TEXT_CHARS).join(''), truncated: true };
  }
  return { text: redacted, truncated: false };
}

function redactSecretLikeText(value: string): string {
  return value
    .split(/\s+/)
    .map((token) => (isSecretLike(token) ? '[redacted]' : token))
    .join(' ');
}

function isSecretLike(value: string): boolean {
  const normalized = value.toLowerCase();
  return (
    normalized.includes('secret') ||
    normalized.includes('token') ||
    normalized.includes('password') ||
    normalized.includes('api_key') ||
    normalized.includes('apikey') ||
    normalized.startsWith('sk-')
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}
