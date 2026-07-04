import { buildTaskMonitor, isWaitingApproval, planProgressText } from '../state/tasks';
import type { DesktopTaskItem } from '../types';

export function TaskList({
  tasks,
  activeTaskId,
  onSelect,
}: {
  tasks: DesktopTaskItem[];
  activeTaskId: string | null;
  onSelect: (taskId: string) => void;
}) {
  if (tasks.length === 0) {
    return <div className="empty-list">No tasks yet</div>;
  }
  const monitor = buildTaskMonitor(tasks);

  return (
    <div className="task-monitor">
      <dl className="task-counts" aria-label="Task monitor counts">
        <dt>Total</dt>
        <dd>{monitor.counts.total}</dd>
        <dt>Active</dt>
        <dd>{monitor.counts.active}</dd>
        <dt>Waiting</dt>
        <dd>{monitor.counts.waiting_approval}</dd>
        <dt>Failed</dt>
        <dd>{monitor.counts.failed}</dd>
      </dl>
      <ul className="task-list" aria-label="Tasks">
        {tasks.map((task) => {
          const progress = planProgressText(task);
          return (
            <li key={task.task_id}>
              <button
                className={task.task_id === activeTaskId ? 'selected' : ''}
                type="button"
                onClick={() => onSelect(task.task_id)}
              >
                <span>{task.summary}</span>
                <small>{task.state}</small>
                {progress !== null ? <small>plan {progress}</small> : null}
                {task.active_todo !== undefined && task.active_todo !== null ? (
                  <small>{task.active_todo.content}</small>
                ) : null}
                {isWaitingApproval(task) ? <small>waiting approval</small> : null}
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
