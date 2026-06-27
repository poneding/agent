import type { DesktopTaskItem } from '../types';

export function TaskDetail({ task }: { task: DesktopTaskItem | null }) {
  if (task === null) {
    return <div className="empty-list">Select a task</div>;
  }

  return (
    <section className="task-detail" aria-label="Task detail">
      <h3>{task.summary}</h3>
      <dl>
        <dt>State</dt>
        <dd>{task.state}</dd>
        <dt>Updated</dt>
        <dd>{task.updated_at}</dd>
      </dl>
    </section>
  );
}
