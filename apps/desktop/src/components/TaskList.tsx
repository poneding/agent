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

  return (
    <ul className="task-list" aria-label="Tasks">
      {tasks.map((task) => (
        <li key={task.task_id}>
          <button
            className={task.task_id === activeTaskId ? 'selected' : ''}
            type="button"
            onClick={() => onSelect(task.task_id)}
          >
            <span>{task.summary}</span>
            <small>{task.state}</small>
          </button>
        </li>
      ))}
    </ul>
  );
}
