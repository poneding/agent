import type { DesktopDashboardSnapshot } from '../types';

export function WorkspaceDashboard({
  snapshot,
  onOpenConversation,
  onOpenApprovals,
}: {
  snapshot: DesktopDashboardSnapshot | null;
  onOpenConversation: () => void;
  onOpenApprovals: () => void;
}) {
  if (snapshot === null) {
    return (
      <section className="workspace-dashboard" aria-label="Dashboard summary">
        <h1>Dashboard</h1>
        <div className="empty-list">Dashboard loading</div>
      </section>
    );
  }

  const connected = snapshot.session.state === 'connected';
  const hasApprovals = snapshot.pending_approval_count > 0;

  return (
    <section className="workspace-dashboard" aria-label="Dashboard summary">
      <div className="dashboard-header">
        <div>
          <h1>Dashboard</h1>
          <p>{snapshot.workspace_root_display ?? snapshot.workspace_id}</p>
        </div>
        <div className={`readiness-pill ${snapshot.session.state}`}>
          {snapshot.session.state}
        </div>
      </div>

      <div className="dashboard-grid">
        <section className="dashboard-panel" aria-label="Session readiness">
          <h2>Session</h2>
          <dl>
            <dt>State</dt>
            <dd>{snapshot.session.state}</dd>
            <dt>Workspace</dt>
            <dd>{snapshot.workspace_id}</dd>
            <dt>Conversation</dt>
            <dd>{snapshot.session.conversation_id ?? 'none'}</dd>
            <dt>Frames</dt>
            <dd>{snapshot.session.pending_frame_count}</dd>
          </dl>
          <button type="button" disabled={!connected} onClick={onOpenConversation}>
            Open conversation
          </button>
        </section>

        <section className="dashboard-panel" aria-label="Provider readiness">
          <h2>Provider</h2>
          <dl>
            <dt>Provider</dt>
            <dd>{snapshot.provider.provider_id}</dd>
            <dt>Model</dt>
            <dd>{snapshot.provider.model_id}</dd>
            <dt>Enabled</dt>
            <dd>{snapshot.provider.enabled ? 'yes' : 'no'}</dd>
            <dt>Diagnostics</dt>
            <dd>{snapshot.provider.diagnostic_count}</dd>
          </dl>
        </section>

        <section className="dashboard-panel" aria-label="Task counts">
          <h2>Tasks</h2>
          <dl>
            <dt>Total</dt>
            <dd>{snapshot.task_counts.total}</dd>
            <dt>Active</dt>
            <dd>{snapshot.task_counts.active}</dd>
            <dt>Approval</dt>
            <dd>{snapshot.task_counts.waiting_approval}</dd>
            <dt>Failed</dt>
            <dd>{snapshot.task_counts.failed}</dd>
            <dt>Terminal</dt>
            <dd>{snapshot.task_counts.terminal}</dd>
          </dl>
        </section>

        <section className="dashboard-panel" aria-label="Approval readiness">
          <h2>Approvals</h2>
          <p className="dashboard-metric">{snapshot.pending_approval_count}</p>
          <button type="button" disabled={!connected || !hasApprovals} onClick={onOpenApprovals}>
            Review approvals
          </button>
        </section>
      </div>

      <section className="dashboard-panel diagnostics-panel" aria-label="Dashboard diagnostics">
        <h2>Diagnostics</h2>
        {snapshot.diagnostics.length === 0 ? (
          <p>No diagnostics</p>
        ) : (
          <ul>
            {snapshot.diagnostics.map((diagnostic) => (
              <li key={`${diagnostic.severity}:${diagnostic.code}`}>
                <strong>{diagnostic.code}</strong>
                <span>{diagnostic.message}</span>
                {diagnostic.truncated ? <small>truncated</small> : null}
              </li>
            ))}
          </ul>
        )}
      </section>
    </section>
  );
}
