export type DesktopRuntimeState =
  | 'not_started'
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'blocked';

export type DesktopRoute = 'conversation' | 'tasks' | 'trace' | 'settings';

export interface DesktopBootstrapDiagnostic {
  severity: string;
  code: string;
  message: string;
}

export interface DesktopBootstrap {
  app_name: string;
  app_version: string;
  runtime_state: DesktopRuntimeState;
  route: DesktopRoute;
  diagnostics: DesktopBootstrapDiagnostic[];
}

export type Timestamp = string;

export interface DesktopConversationItem {
  conversation_id: string;
  title: string;
  last_activity_at: Timestamp;
  unread_count: number;
}

export type DesktopTaskState =
  | 'created'
  | 'queued'
  | 'running'
  | 'waiting_approval'
  | 'sleeping'
  | 'paused'
  | 'succeeded'
  | 'failed'
  | 'cancelled'
  | 'expired';

export interface DesktopTaskItem {
  task_id: string;
  workspace_id: string;
  conversation_id?: string | null;
  state: DesktopTaskState;
  summary: string;
  updated_at: Timestamp;
}

export interface DesktopWorkspaceSnapshot {
  workspace_id: string;
  active_conversation_id?: string | null;
  conversations: DesktopConversationItem[];
  tasks: DesktopTaskItem[];
}

export interface DesktopStartSessionRequest {
  workspace_path?: string | null;
  client_version: string;
}

export interface DesktopSessionSnapshot {
  session_id: string;
  workspace_id: string;
  conversation_id: string;
  state: DesktopRuntimeState;
  pending_frame_count: number;
}

export interface DesktopCommandAck {
  command_id: string;
  accepted: boolean;
  state: DesktopRuntimeState;
  pending_frame_count: number;
}

export interface DesktopBridgeError {
  code: string;
  message: string;
  recoverable: boolean;
}

export interface DesktopResolveApprovalRequest {
  request_id: string;
  decision: 'allow' | 'deny';
  reason: string;
  lease_duration_seconds?: number | null;
}

export type DesktopBridgeConnectionState =
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'blocked';

export interface UiClientInfo {
  kind: 'cli' | 'tui' | 'desktop' | 'api' | 'test';
  version: string;
}

export type UiApprovalDecision = 'allow' | 'deny';

export type UiCommand =
  | {
      type: 'start_turn';
      data: {
        workspace_id: string;
        conversation_id: string;
        prompt: string;
      };
    }
  | {
      type: 'cancel_turn';
      data: {
        conversation_id: string;
        turn_id: string;
        reason: string;
      };
    }
  | {
      type: 'permission_response';
      data: {
        request_id: string;
        decision: UiApprovalDecision;
        reason: string;
      };
    }
  | {
      type: 'resolve_approval';
      data: {
        request_id: string;
        decision: UiApprovalDecision;
        reason: string;
        lease_duration_seconds?: number | null;
      };
    }
  | {
      type: 'trace_request';
      data: {
        conversation_id: string;
        include_tools: boolean;
        include_policy: boolean;
        include_checkpoints: boolean;
        include_final: boolean;
      };
    }
  | {
      type: 'ping';
    };

export interface UiCommandEnvelope {
  schema_version: number;
  command_id: string;
  client: UiClientInfo;
  command: UiCommand;
}

export interface UiApprovalPreviewSummary {
  kind: string;
  resource: string;
  summary: string;
  content_hash?: string | null;
  truncated: boolean;
  redacted: boolean;
}

export interface UiApprovalRequest {
  request_id: string;
  status: string;
  action: string;
  resource_kind: string;
  resource: string;
  reason: string;
  preview: UiApprovalPreviewSummary;
  max_duration_seconds?: number | null;
  expires_at?: string | null;
}

export interface ApprovalUiState {
  pending: UiApprovalRequest[];
  selectedRequestId: string | null;
  lastDecisionStatus: string | null;
}

export type UiJsonFrame =
  | {
      type: 'error';
      data: {
        code: string;
        message: string;
        recoverable: boolean;
      };
    }
  | {
      type: 'done';
      data: {
        event_count: number;
      };
    }
  | {
      type: string;
      data?: unknown;
    };

export interface TranscriptEntry {
  id: string;
  kind: 'user' | 'assistant' | 'diagnostic' | 'progress';
  content: string;
}

export interface ConversationTaskState {
  workspaceId: string;
  conversations: DesktopConversationItem[];
  tasks: DesktopTaskItem[];
  transcript: TranscriptEntry[];
  activeConversationId: string | null;
  activeTaskId: string | null;
  composerText: string;
}
