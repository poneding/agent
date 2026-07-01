import type { DesktopTaskItem } from '../types';
import { isWaitingApproval, planProgressText } from '../state/tasks';

export function TaskDetail({ task }: { task: DesktopTaskItem | null }) {
  if (task === null) {
    return <div className="empty-list">Select a task</div>;
  }
  const progress = planProgressText(task);

  return (
    <section className="task-detail" aria-label="Task detail">
      <h3>{task.summary}</h3>
      <dl>
        <dt>State</dt>
        <dd>{task.state}</dd>
        <dt>Updated</dt>
        <dd>{task.updated_at}</dd>
        <dt>Latest</dt>
        <dd>{task.latest_run_summary ?? 'not run'}</dd>
        <dt>Plan</dt>
        <dd>{progress ?? 'none'}</dd>
        <dt>Active Todo</dt>
        <dd>{task.active_todo?.content ?? 'none'}</dd>
        <dt>Waiting</dt>
        <dd>{isWaitingApproval(task) ? 'yes' : 'no'}</dd>
        <dt>Approval</dt>
        <dd>{task.waiting_approval_id ?? task.latest_notification_approval_id ?? 'none'}</dd>
        <dt>Notification</dt>
        <dd>{task.latest_notification ?? 'none'}</dd>
        <dt>Notification Kind</dt>
        <dd>{task.latest_notification_kind ?? 'none'}</dd>
        <dt>Artifacts</dt>
        <dd>{task.artifact_count ?? 0}</dd>
      </dl>
    </section>
  );
}
