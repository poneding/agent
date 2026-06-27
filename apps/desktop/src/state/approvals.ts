import type {
  ApprovalUiState,
  UiApprovalDecision,
  UiApprovalRequest,
  UiCommandEnvelope,
  UiJsonFrame,
} from '../types';

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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}
