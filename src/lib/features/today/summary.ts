// What «Hoy» shows about a day (F20-01): the strip of coming days and the summary banner.
// Pure functions over the tasks so the view only derives from them.
import type { Task } from "../../types";
import { addDays } from "../calendar/grid";

/** How many days the strip spans, today included. */
export const STRIP_DAYS = 14;

/** The dates of the strip, starting today. */
export function stripDates(today: string, count = STRIP_DAYS): string[] {
  return Array.from({ length: count }, (_, index) => addDays(today, index));
}

/** Open tasks per due date, for the marks under each day of the strip. */
export function pendingByDate(tasks: readonly Task[]): Map<string, number> {
  const counts = new Map<string, number>();
  for (const task of tasks) {
    if (task.isCompleted || task.dueDate === null) continue;
    counts.set(task.dueDate, (counts.get(task.dueDate) ?? 0) + 1);
  }
  return counts;
}

export interface DaySummary {
  total: number;
  done: number;
  pending: number;
  /** Open tasks from days before today; only counted when the day shown is today. */
  overdue: number;
}

export function summarizeDay(
  tasks: readonly Task[],
  date: string,
  today: string,
): DaySummary {
  let total = 0;
  let done = 0;
  let overdue = 0;
  for (const task of tasks) {
    if (task.dueDate === date) {
      total += 1;
      if (task.isCompleted) done += 1;
    } else if (
      date === today &&
      !task.isCompleted &&
      task.dueDate !== null &&
      task.dueDate < today
    ) {
      overdue += 1;
    }
  }
  return { total, done, pending: total - done, overdue };
}
