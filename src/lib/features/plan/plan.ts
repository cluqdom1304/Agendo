// «Planificar mi día» (F24-02): which tasks could go into today and what the day adds up to.
// Pure functions over the tasks, so the dialog only derives from them.
import type { Task } from "../../types";

/** A day with more estimated work than this is flagged as overloaded (8 hours). */
export const CAPACITY_MINUTES = 8 * 60;

export interface PlanCandidates {
  /** Open tasks from days before today, oldest first. */
  overdue: Task[];
  /** Open tasks without a date, the ones waiting in the inbox. */
  undated: Task[];
  /** Open tasks already due today. */
  today: Task[];
}

export function planCandidates(
  tasks: readonly Task[],
  today: string,
): PlanCandidates {
  const open = tasks.filter((task) => !task.isCompleted);
  return {
    overdue: open
      .filter((task) => task.dueDate !== null && task.dueDate < today)
      .sort((a, b) => (a.dueDate ?? "").localeCompare(b.dueDate ?? "")),
    undated: open.filter((task) => task.dueDate === null),
    today: open.filter((task) => task.dueDate === today),
  };
}

export interface PlanTotals {
  tasks: number;
  /** Sum of the durations that are known. */
  minutes: number;
  /** Tasks with no estimate, which the sum cannot count. */
  unestimated: number;
  overloaded: boolean;
}

export function planTotals(
  tasks: readonly Task[],
  capacity = CAPACITY_MINUTES,
): PlanTotals {
  let minutes = 0;
  let unestimated = 0;
  for (const task of tasks) {
    if (task.durationMinutes === null) unestimated += 1;
    else minutes += task.durationMinutes;
  }
  return {
    tasks: tasks.length,
    minutes,
    unestimated,
    overloaded: minutes > capacity,
  };
}
