import type { UiApprovalRequest } from '../types';

export function ApprovalQueue({
  approvals,
  selectedRequestId,
  onSelect,
}: {
  approvals: UiApprovalRequest[];
  selectedRequestId: string | null;
  onSelect: (requestId: string) => void;
}) {
  if (approvals.length === 0) {
    return <div className="empty-list">No pending approvals</div>;
  }

  return (
    <ul className="approval-queue" aria-label="Pending approvals">
      {approvals.map((approval) => (
        <li key={approval.request_id}>
          <button
            className={approval.request_id === selectedRequestId ? 'selected' : ''}
            type="button"
            onClick={() => onSelect(approval.request_id)}
          >
            <span>{approval.resource}</span>
            <small>
              {approval.action} - {approval.status}
            </small>
          </button>
        </li>
      ))}
    </ul>
  );
}
