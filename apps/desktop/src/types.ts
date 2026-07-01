export type DesktopRuntimeState =
  | 'not_started'
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'blocked';

export type DesktopRoute = 'dashboard' | 'conversation' | 'tasks' | 'trace' | 'settings';
export type DesktopShellRoute = DesktopRoute | 'approvals';

export interface DesktopBootstrapDiagnostic {
  severity: string;
  code: string;
  message: string;
}

export type TraceTimelineSeverity = 'info' | 'warning' | 'error';

export interface TraceTimelineItem {
  seq: number;
  kind: string;
  title: string;
  summary: string;
  severity: TraceTimelineSeverity;
}

export interface TraceTimeline {
  conversation_id: string;
  items: TraceTimelineItem[];
}

export interface TraceUiState {
  conversationId: string | null;
  timeline: TraceTimeline | null;
  loading: boolean;
  error: string | null;
}

export interface UiModelProviderSummary {
  id: string;
  kind: string;
  default_model: string;
  enabled: boolean;
}

export interface UiConfigDiagnostic {
  severity: 'info' | 'warning' | 'error';
  code: string;
  message: string;
  field_path?: string | null;
  source?: string | null;
}

export interface UiConfigSnapshot {
  selected_provider: string;
  selected_model: string;
  providers: UiModelProviderSummary[];
  diagnostics: UiConfigDiagnostic[];
}

export interface DesktopConfigSnapshot {
  snapshot: UiConfigSnapshot;
}

export interface UiExtensionSummary {
  id: string;
  kind: string;
  enabled: boolean;
  health: string;
  version?: string | null;
  source_label: string;
  capability_count: number;
}

export interface UiExtensionCapabilityReview {
  extension_id: string;
  capability_kind: string;
  resource: string;
  reason: string;
}

export interface UiExtensionDiagnostic {
  severity: string;
  code: string;
  message: string;
  extension_id?: string | null;
  truncated: boolean;
}

export interface UiExtensionSnapshot {
  extensions: UiExtensionSummary[];
  capability_reviews: UiExtensionCapabilityReview[];
  diagnostics: UiExtensionDiagnostic[];
}

export type DesktopTheme = 'system' | 'light' | 'dark';

export interface DesktopSettings {
  model: {
    provider: string;
    model: string;
  };
  policy: {
    default_decision: string;
    require_approval_for_shell: boolean;
  };
  memory: {
    project_memory_enabled: boolean;
    long_term_memory_enabled: boolean;
  };
  mcp: {
    enabled: boolean;
    server_count: number;
  };
  ui: {
    theme: DesktopTheme;
    show_trace_timeline: boolean;
  };
}

export interface SettingsUiState {
  snapshot: DesktopConfigSnapshot | null;
  settings: DesktopSettings;
  validationDiagnostics: DesktopBootstrapDiagnostic[];
  loading: boolean;
  error: string | null;
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

export type DesktopSessionReadinessState =
  | 'not_started'
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'blocked';

export interface DesktopSessionReadiness {
  state: DesktopSessionReadinessState;
  session_id?: string | null;
  conversation_id?: string | null;
  pending_frame_count: number;
}

export interface DesktopProviderReadiness {
  provider_id: string;
  model_id: string;
  enabled: boolean;
  diagnostic_count: number;
}

export interface DesktopTaskCounts {
  total: number;
  active: number;
  waiting_approval: number;
  failed: number;
  terminal: number;
}

export type DesktopDashboardDiagnosticSeverity = 'info' | 'warning' | 'error';

export interface DesktopDashboardDiagnostic {
  severity: DesktopDashboardDiagnosticSeverity;
  code: string;
  message: string;
  truncated: boolean;
}

export interface DesktopDashboardSnapshot {
  workspace_id: string;
  workspace_root_display?: string | null;
  session: DesktopSessionReadiness;
  provider: DesktopProviderReadiness;
  task_counts: DesktopTaskCounts;
  pending_approval_count: number;
  diagnostics: DesktopDashboardDiagnostic[];
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

export type ApprovalDecisionStatus = 'idle' | 'accepted' | 'rejected' | 'error';

export interface ApprovalUiState {
  pending: UiApprovalRequest[];
  selectedRequestId: string | null;
  lastDecisionStatus: string | null;
  lastDecisionAck: DesktopCommandAck | null;
  lastDecisionError: BridgeErrorNotice | null;
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
      type: 'extension_snapshot';
      data: UiExtensionSnapshot;
    }
  | {
      type: string;
      data?: unknown;
    };

export interface TranscriptEntry {
  id: string;
  kind: 'user' | 'assistant' | 'diagnostic' | 'progress' | 'system';
  content: string;
  truncated?: boolean;
}

export type TurnSubmissionStatus =
  | 'draft'
  | 'submitting'
  | 'accepted'
  | 'rejected'
  | 'draining_frames'
  | 'completed';

export interface TurnCommandAck {
  command_id: string;
  accepted: boolean;
  pending_frame_count: number;
}

export interface BridgeErrorNotice {
  code: string;
  message: string;
  recoverable: boolean;
  truncated: boolean;
}

export interface TurnSubmission {
  command_id: string;
  conversation_id: string;
  prompt_preview: string;
  status: TurnSubmissionStatus;
  ack: TurnCommandAck | null;
  error: BridgeErrorNotice | null;
  truncated: boolean;
}

export interface ConversationDiagnostic {
  severity: 'info' | 'warning' | 'error';
  code: string;
  message: string;
  truncated: boolean;
}

export type DesktopDiffLineKind = 'context' | 'addition' | 'removal';

export interface DesktopDiffLinePreview {
  kind: DesktopDiffLineKind;
  old_line?: number | null;
  new_line?: number | null;
  content: string;
  truncated: boolean;
}

export interface DesktopDiffHunkPreview {
  old_start: number;
  new_start: number;
  lines: DesktopDiffLinePreview[];
  truncated: boolean;
}

export interface DesktopDiffFileReview {
  path: string;
  added_lines: number;
  removed_lines: number;
  summary: string;
  hunks: DesktopDiffHunkPreview[];
}

export interface DesktopDiffReview {
  review_id: string;
  status: 'pending' | 'approved' | 'rejected';
  files: DesktopDiffFileReview[];
  selected_path?: string | null;
  decision_enabled: boolean;
  disabled_reason?: string | null;
}

export interface ConversationTaskState {
  workspaceId: string;
  conversations: DesktopConversationItem[];
  tasks: DesktopTaskItem[];
  transcript: TranscriptEntry[];
  activeConversationId: string | null;
  activeTaskId: string | null;
  composerText: string;
  currentSubmission: TurnSubmission | null;
  diagnostics: ConversationDiagnostic[];
  pendingFrameCount: number;
}
