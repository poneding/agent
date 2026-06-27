import { FormEvent, useState } from 'react';

import type { UiApprovalRequest } from '../types';
import { DiffPreview } from './DiffPreview';

export function ApprovalDetail({
  approval,
  onAllow,
  onDeny,
}: {
  approval: UiApprovalRequest | null;
  onAllow: (requestId: string) => void;
  onDeny: (requestId: string, reason: string) => void;
}) {
  const [denyReason, setDenyReason] = useState('');
  const [error, setError] = useState<string | null>(null);

  if (approval === null) {
    return <div className="empty-list">Select an approval</div>;
  }
  const currentApproval = approval;

  function handleDeny(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
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
        <dt>Action</dt>
        <dd>{currentApproval.action}</dd>
        <dt>Reason</dt>
        <dd>{currentApproval.reason}</dd>
      </dl>
      <DiffPreview preview={currentApproval.preview} />
      <div className="approval-actions">
        <button type="button" onClick={() => onAllow(currentApproval.request_id)}>
          Allow
        </button>
        <form onSubmit={handleDeny}>
          <input
            aria-label="Deny reason"
            value={denyReason}
            onChange={(event) => setDenyReason(event.currentTarget.value)}
          />
          <button type="submit">Deny</button>
        </form>
      </div>
      {error ? (
        <p className="inline-error" role="alert">
          {error}
        </p>
      ) : null}
    </section>
  );
}
