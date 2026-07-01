import type {
  ApprovalUiState,
  BridgeErrorNotice,
  DesktopBridgeError,
  DesktopCommandAck,
  UiApprovalDecision,
  UiApprovalRequest,
  UiCommandEnvelope,
  UiJsonFrame,
} from '../types';

const MAX_APPROVAL_TEXT_CHARS = 240;

interface ResolveApprovalInput {
  requestId: string;
  decision: UiApprovalDecision;
  reason: string;
  clientVersion: string;
  leaseDurationSeconds?: number | null;
}

export function createApprovalState(): ApprovalUiState {
  return {
    pending: [],
    selectedRequestId: null,
    lastDecisionStatus: null,
    lastDecisionAck: null,
    lastDecisionError: null,
  };
}

export function reduceApprovalState(
  state: ApprovalUiState,
  frame: UiJsonFrame,
): ApprovalUiState {
  if (frame.type !== 'event' || !isRecord(frame.data)) {
    return state;
  }

  const payload = frame.data.payload;
  if (!isRecord(payload) || typeof payload.type !== 'string') {
    return state;
  }

  if (payload.type === 'approval_requested' && isApprovalRequest(payload.data)) {
    return upsertApproval(state, payload.data);
  }

  if (payload.type === 'approval_resolved' && isRecord(payload.data)) {
    const requestId = stringField(payload.data, 'request_id');
    if (requestId === null) {
      return state;
    }
    return removeApproval(state, requestId, stringField(payload.data, 'status'));
  }

  return state;
}

export function createResolveApprovalEnvelope(input: ResolveApprovalInput): UiCommandEnvelope {
  return {
    schema_version: 1,
    command_id: createCommandId(),
    client: {
      kind: 'desktop',
      version: input.clientVersion,
    },
    command: {
      type: 'resolve_approval',
      data: {
        request_id: input.requestId,
        decision: input.decision,
        reason: input.reason,
        lease_duration_seconds: input.leaseDurationSeconds ?? null,
      },
    },
  };
}

export function selectApproval(
  state: ApprovalUiState,
  requestId: string,
): ApprovalUiState {
  if (!state.pending.some((approval) => approval.request_id === requestId)) {
    return state;
  }
  return {
    ...state,
    selectedRequestId: requestId,
  };
}

export function reduceApprovalCommandAck(
  state: ApprovalUiState,
  ack: DesktopCommandAck,
): ApprovalUiState {
  return {
    ...state,
    lastDecisionStatus: ack.accepted ? 'accepted' : 'rejected',
    lastDecisionAck: ack,
    lastDecisionError: null,
  };
}

export function reduceApprovalError(
  state: ApprovalUiState,
  error: DesktopBridgeError | BridgeErrorNotice,
): ApprovalUiState {
  const notice = bridgeErrorNotice(error);
  return {
    ...state,
    lastDecisionStatus: 'error',
    lastDecisionAck: null,
    lastDecisionError: notice,
  };
}

export function approvalDecisionDisabledReason(approval: UiApprovalRequest): string | null {
  const status = approval.status.trim().toLowerCase();
  if (status === 'pending') {
    if (approval.expires_at !== null && approval.expires_at !== undefined) {
      const expiresAt = Date.parse(approval.expires_at);
      if (!Number.isNaN(expiresAt) && expiresAt <= Date.now()) {
        return 'approval expired';
      }
    }
    return null;
  }
  if (status === 'allowed' || status === 'approved') {
    return 'approval already allowed';
  }
  if (status === 'denied' || status === 'rejected') {
    return 'approval already denied';
  }
  if (status === 'expired') {
    return 'approval expired';
  }
  return 'approval already resolved';
}

function upsertApproval(
  state: ApprovalUiState,
  approval: UiApprovalRequest,
): ApprovalUiState {
  const existingIndex = state.pending.findIndex(
    (item) => item.request_id === approval.request_id,
  );
  const pending =
    existingIndex >= 0
      ? state.pending.map((item, index) => (index === existingIndex ? approval : item))
      : [approval, ...state.pending];

  return {
    ...state,
    pending,
    selectedRequestId: state.selectedRequestId ?? approval.request_id,
  };
}

function removeApproval(
  state: ApprovalUiState,
  requestId: string,
  status: string | null,
): ApprovalUiState {
  const pending = state.pending.filter((approval) => approval.request_id !== requestId);
  return {
    pending,
    selectedRequestId:
      state.selectedRequestId === requestId ? pending[0]?.request_id ?? null : state.selectedRequestId,
    lastDecisionStatus: status,
    lastDecisionAck: state.lastDecisionAck,
    lastDecisionError: state.lastDecisionError,
  };
}

function isApprovalRequest(value: unknown): value is UiApprovalRequest {
  return (
    isRecord(value) &&
    typeof value.request_id === 'string' &&
    typeof value.status === 'string' &&
    typeof value.action === 'string' &&
    typeof value.resource_kind === 'string' &&
    typeof value.resource === 'string' &&
    typeof value.reason === 'string' &&
    isRecord(value.preview) &&
    typeof value.preview.kind === 'string' &&
    typeof value.preview.resource === 'string' &&
    typeof value.preview.summary === 'string' &&
    typeof value.preview.truncated === 'boolean' &&
    typeof value.preview.redacted === 'boolean'
  );
}

function stringField(value: Record<string, unknown>, field: string): string | null {
  const fieldValue = value[field];
  return typeof fieldValue === 'string' ? fieldValue : null;
}

function createCommandId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `desktop-approval-${Date.now()}`;
}

function bridgeErrorNotice(error: DesktopBridgeError | BridgeErrorNotice): BridgeErrorNotice {
  const code = boundApprovalText(error.code);
  const message = boundApprovalText(error.message);
  return {
    code: code.text || 'desktop.approval.error',
    message: message.text || 'desktop approval error',
    recoverable: error.recoverable,
    truncated: code.truncated || message.truncated || ('truncated' in error && error.truncated),
  };
}

function boundApprovalText(value: string): { text: string; truncated: boolean } {
  const redacted = redactSecretLikeText(value).trim();
  const chars = Array.from(redacted);
  if (chars.length > MAX_APPROVAL_TEXT_CHARS) {
    return { text: chars.slice(0, MAX_APPROVAL_TEXT_CHARS).join(''), truncated: true };
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
