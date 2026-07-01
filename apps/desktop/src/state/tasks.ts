import type { DesktopTaskItem, DesktopTaskMonitor, DesktopTaskMonitorCounts } from '../types';

export function selectTask(
  tasks: DesktopTaskItem[],
  taskId: string | null,
): DesktopTaskItem | null {
  if (taskId === null) {
    return null;
  }
  return tasks.find((task) => task.task_id === taskId) ?? null;
}

export function buildTaskMonitor(tasks: DesktopTaskItem[]): DesktopTaskMonitor {
  return {
    counts: countTasks(tasks),
    tasks,
  };
}

function countTasks(tasks: DesktopTaskItem[]): DesktopTaskMonitorCounts {
  const counts: DesktopTaskMonitorCounts = {
    total: tasks.length,
    active: 0,
    waiting_approval: 0,
    failed: 0,
    terminal: 0,
  };

  for (const task of tasks) {
    if (isTerminal(task.state)) {
      counts.terminal += 1;
    }
    if (task.state === 'waiting_approval') {
      counts.waiting_approval += 1;
    } else if (task.state === 'failed') {
      counts.failed += 1;
    } else if (!isTerminal(task.state)) {
      counts.active += 1;
    }
  }

  return counts;
}

function isTerminal(state: DesktopTaskItem['state']): boolean {
  return (
    state === 'succeeded' ||
    state === 'failed' ||
    state === 'cancelled' ||
    state === 'expired'
  );
}
