import { FormEvent, useState } from 'react';

import { approvalDecisionDisabledReason } from '../state/approvals';
import type { BridgeErrorNotice, DesktopCommandAck, UiApprovalRequest } from '../types';
import { DiffPreview } from './DiffPreview';

export function ApprovalDetail({
  approval,
  decisionAck = null,
  decisionError = null,
  onAllow,
  onDeny,
}: {
  approval: UiApprovalRequest | null;
  decisionAck?: DesktopCommandAck | null;
  decisionError?: BridgeErrorNotice | null;
  onAllow: (requestId: string) => void;
  onDeny: (requestId: string, reason: string) => void;
}) {
  const [denyReason, setDenyReason] = useState('');
  const [error, setError] = useState<string | null>(null);

  if (approval === null) {
    return <div className="empty-list">Select an approval</div>;
  }
  const currentApproval = approval;
  const disabledReason = approvalDecisionDisabledReason(currentApproval);
  const decisionDisabled = disabledReason !== null;

  function handleDeny(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (decisionDisabled) {
      return;
    }
    const reason = denyReason.trim();
    if (reason.length === 0) {
      setError('Deny reason is required');
      return;
    }
    setError(null);
    onDeny(currentApproval.request_id, reason);
  }

  return (
    <section className="approval-detail" aria-label="Approval detail">
      <h3>{currentApproval.resource}</h3>
      <dl>
        <dt>Status</dt>
        <dd>{currentApproval.status}</dd>
        <dt>Action</dt>
        <dd>{currentApproval.action}</dd>
        <dt>Resource type</dt>
        <dd>{currentApproval.resource_kind}</dd>
        <dt>Reason</dt>
        <dd>{currentApproval.reason}</dd>
        <dt>Lease</dt>
        <dd>
          {currentApproval.max_duration_seconds === null ||
          currentApproval.max_duration_seconds === undefined
            ? 'none'
            : `${currentApproval.max_duration_seconds}s`}
        </dd>
        <dt>Expires</dt>
        <dd>{currentApproval.expires_at ?? 'not set'}</dd>
      </dl>
      {disabledReason === null ? null : (
        <p className="approval-disabled">{disabledReason}</p>
      )}
      <DiffPreview preview={currentApproval.preview} />
      <div className="approval-actions">
        <button
          disabled={decisionDisabled}
          type="button"
          onClick={() => onAllow(currentApproval.request_id)}
        >
          Allow
        </button>
        <form onSubmit={handleDeny}>
          <input
            aria-label="Deny reason"
            disabled={decisionDisabled}
            value={denyReason}
            onChange={(event) => setDenyReason(event.currentTarget.value)}
          />
          <button disabled={decisionDisabled} type="submit">
            Deny
          </button>
        </form>
      </div>
      {decisionAck === null ? null : (
        <p className="approval-ack" role="status">
          {decisionAck.accepted ? 'decision accepted' : 'decision rejected'} -{' '}
          {decisionAck.pending_frame_count} pending frames
        </p>
      )}
      {decisionError === null ? null : (
        <p className="inline-error" role="alert">
          {decisionError.code}: {decisionError.message}
        </p>
      )}
      {error ? (
        <p className="inline-error" role="alert">
          {error}
        </p>
      ) : null}
    </section>
  );
}
