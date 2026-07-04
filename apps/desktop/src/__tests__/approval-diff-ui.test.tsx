import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { ApprovalDetail } from '../components/ApprovalDetail';
import { ApprovalQueue } from '../components/ApprovalQueue';
import { DiffPreview } from '../components/DiffPreview';
import {
  approvalDecisionDisabledReason,
  createApprovalState,
  createResolveApprovalEnvelope,
  reduceApprovalCommandAck,
  reduceApprovalError,
  reduceApprovalState,
} from '../state/approvals';
import type { DesktopDiffReview, UiApprovalRequest, UiJsonFrame } from '../types';

const approval: UiApprovalRequest = {
  request_id: 'approval-1',
  status: 'pending',
  action: 'write',
  resource_kind: 'file',
  resource: 'src/main.rs',
  reason: 'edit file',
  preview: {
    kind: 'file_edit',
    resource: 'src/main.rs',
    summary: '+hello\n-world',
    content_hash: 'hash-1',
    truncated: true,
    redacted: true,
  },
  max_duration_seconds: 120,
  expires_at: null,
};

const visualDiffReview: DesktopDiffReview = {
  review_id: 'review-1',
  status: 'pending',
  selected_path: 'src/main.rs',
  decision_enabled: true,
  disabled_reason: null,
  files: [
    {
      path: 'src/main.rs',
      added_lines: 1,
      removed_lines: 1,
      summary: 'Update main entrypoint',
      hunks: [
        {
          old_start: 3,
          new_start: 3,
          truncated: true,
          lines: [
            {
              kind: 'removal',
              old_line: 3,
              new_line: null,
              content: '-old line',
              truncated: false,
            },
            {
              kind: 'addition',
              old_line: null,
              new_line: 3,
              content: '+new line [redacted]',
              truncated: true,
            },
          ],
        },
      ],
    },
  ],
};

function approvalFrame(request: UiApprovalRequest): UiJsonFrame {
  return {
    type: 'event',
    data: {
      payload: {
        type: 'approval_requested',
        data: request,
      },
    },
  };
}

function resolvedFrame(requestId: string, status = 'resolved'): UiJsonFrame {
  return {
    type: 'event',
    data: {
      payload: {
        type: 'approval_resolved',
        data: {
          request_id: requestId,
          status,
          decision: 'allow',
          reason: 'done',
        },
      },
    },
  };
}

describe('approval diff UI', () => {
  afterEach(() => cleanup());

  it('queues approvals and removes resolved approvals', () => {
    let state = createApprovalState();
    state = reduceApprovalState(state, approvalFrame(approval));

    expect(state.pending).toHaveLength(1);
    expect(state.selectedRequestId).toBe('approval-1');

    state = reduceApprovalState(state, resolvedFrame('approval-1', 'consumed'));

    expect(state.pending).toHaveLength(0);
    expect(state.selectedRequestId).toBeNull();
    expect(state.lastDecisionStatus).toBe('consumed');
    expect(state.lastDecisionAck).toBeNull();
  });

  it('tracks approval command acknowledgements and recoverable errors', () => {
    let state = createApprovalState();
    state = reduceApprovalCommandAck(state, {
      command_id: 'command-1',
      accepted: true,
      state: 'connected',
      pending_frame_count: 2,
    });

    expect(state.lastDecisionStatus).toBe('accepted');
    expect(state.lastDecisionAck?.pending_frame_count).toBe(2);

    state = reduceApprovalError(state, {
      code: 'desktop.approval.invalid',
      message: 'token sk-secret password',
      recoverable: true,
    });

    expect(state.lastDecisionStatus).toBe('error');
    expect(state.lastDecisionError?.message).toContain('[redacted]');
    expect(state.lastDecisionError?.message).not.toContain('sk-secret');
  });

  it('renders queue, detail, redaction, and truncated diff preview', () => {
    const allow = vi.fn();
    const deny = vi.fn();

    render(
      <>
        <ApprovalQueue
          approvals={[approval]}
          selectedRequestId="approval-1"
          onSelect={() => undefined}
        />
        <ApprovalDetail
          approval={approval}
          decisionAck={{
            command_id: 'command-1',
            accepted: true,
            state: 'connected',
            pending_frame_count: 1,
          }}
          onAllow={allow}
          onDeny={deny}
        />
      </>,
    );

    expect(within(screen.getByLabelText('Pending approvals')).getByText('src/main.rs')).toBeInTheDocument();
    expect(screen.getByText('redacted')).toBeInTheDocument();
    expect(screen.getByText('truncated')).toBeInTheDocument();
    expect(screen.getByText('decision accepted - 1 pending frames')).toBeInTheDocument();
    expect(screen.getByText('+hello')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Allow' }));
    fireEvent.change(screen.getByLabelText('Deny reason'), {
      target: { value: 'unsafe' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Deny' }));

    expect(allow).toHaveBeenCalledWith('approval-1');
    expect(deny).toHaveBeenCalledWith('approval-1', 'unsafe');
  });

  it('disables terminal approval decisions with a typed reason', () => {
    const expired = { ...approval, status: 'expired' };
    const allow = vi.fn();
    const deny = vi.fn();

    render(<ApprovalDetail approval={expired} onAllow={allow} onDeny={deny} />);

    expect(approvalDecisionDisabledReason(expired)).toBe('approval expired');
    expect(screen.getByText('approval expired')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Allow' })).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Deny' })).toBeDisabled();
  });

  it('requires deny reason before emitting denial', () => {
    const deny = vi.fn();
    render(<ApprovalDetail approval={approval} onAllow={vi.fn()} onDeny={deny} />);

    fireEvent.click(screen.getByRole('button', { name: 'Deny' }));

    expect(deny).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toHaveTextContent('Deny reason is required');
  });

  it('builds typed resolve approval commands', () => {
    const envelope = createResolveApprovalEnvelope({
      requestId: 'approval-1',
      decision: 'deny',
      reason: 'unsafe',
      clientVersion: '0.1.0',
      leaseDurationSeconds: null,
    });

    expect(envelope.command).toEqual({
      type: 'resolve_approval',
      data: {
        request_id: 'approval-1',
        decision: 'deny',
        reason: 'unsafe',
        lease_duration_seconds: null,
      },
    });
  });

  it('renders a diff preview safely without secret expansion', () => {
    render(<DiffPreview preview={approval.preview} review={visualDiffReview} />);

    expect(screen.getByText('file_edit')).toBeInTheDocument();
    expect(screen.getByText('redacted')).toBeInTheDocument();
    expect(screen.getAllByText('src/main.rs')).toHaveLength(2);
    expect(screen.getByText('Update main entrypoint')).toBeInTheDocument();
    expect(screen.getByText('-old line')).toBeInTheDocument();
    expect(screen.getByText('+new line [redacted] [truncated]')).toBeInTheDocument();
    expect(screen.queryByText('sk-secret')).not.toBeInTheDocument();
  });
});
