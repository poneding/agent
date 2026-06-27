import type {
  ConversationTaskState,
  DesktopTaskItem,
  DesktopTaskState,
  DesktopWorkspaceSnapshot,
  TranscriptEntry,
  UiCommandEnvelope,
  UiJsonFrame,
} from '../types';

interface StartTurnInput {
  workspaceId: string;
  conversationId: string;
  prompt: string;
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
    activeConversationId: snapshot.active_conversation_id ?? null,
    activeTaskId: snapshot.tasks[0]?.task_id ?? null,
    composerText: '',
  };
}

export function reduceConversationTaskState(
  state: ConversationTaskState,
  frame: UiJsonFrame,
): ConversationTaskState {
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

  const transcriptEntry = transcriptEntryFromPayload(payload.type, payload.data, state.transcript);
  if (transcriptEntry === null) {
    return state;
  }

  return {
    ...state,
    transcript: [...state.transcript, transcriptEntry],
  };
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

function transcriptEntry(
  transcript: TranscriptEntry[],
  kind: TranscriptEntry['kind'],
  content: string,
): TranscriptEntry {
  return {
    id: `entry-${transcript.length + 1}`,
    kind,
    content,
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

function createCommandId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `desktop-command-${Date.now()}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}
