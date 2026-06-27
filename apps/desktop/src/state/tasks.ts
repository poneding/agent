import type { DesktopTaskItem } from '../types';

export function selectTask(
  tasks: DesktopTaskItem[],
  taskId: string | null,
): DesktopTaskItem | null {
  if (taskId === null) {
    return null;
  }
  return tasks.find((task) => task.task_id === taskId) ?? null;
}
