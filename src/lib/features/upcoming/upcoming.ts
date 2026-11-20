// What «Próximas» shows (F20-05): the open tasks still to do, by day. Pure so the view only
// derives from it.
import { sortDay } from "../../ordering";
import type { Task } from "../../types";
import { addDays } from "../calendar/grid";

/** Days ahead, today included, that get a heading of their own. */
export const UPCOMING_DAYS = 30;

export interface UpcomingGroups {
  /** Open tasks from days before today, oldest day first. */
  overdue: Task[];
  /** Days within the horizon that have open tasks, earliest first. */
  days: { date: string; tasks: Task[] }[];
  /** Open tasks after the horizon, by date. */
  later: Task[];
}

export function upcomingGroups(
  tasks: readonly Task[],
  today: string,
  horizon = UPCOMING_DAYS,
): UpcomingGroups {
  const last = addDays(today, horizon - 1);
  const overdue: Task[] = [];
  const later: Task[] = [];
  const soon: Task[] = [];
  for (const task of tasks) {
    if (task.isCompleted || task.dueDate === null) continue;
    if (task.dueDate < today) overdue.push(task);
    else if (task.dueDate > last) later.push(task);
    else soon.push(task);
  }
  return {
    overdue: groups(overdue).flatMap((day) => day.tasks),
    days: groups(soon),
    later: groups(later).flatMap((day) => day.tasks),
  };
}

/** Tasks by day, earliest first, each day in the list's order. */
function groups(tasks: readonly Task[]): { date: string; tasks: Task[] }[] {
  const byDate = new Map<string, Task[]>();
  for (const task of tasks) {
    const date = task.dueDate ?? "";
    const day = byDate.get(date);
    if (day) day.push(task);
    else byDate.set(date, [task]);
  }
  return [...byDate]
    .sort(([a], [b]) => (a < b ? -1 : 1))
    .map(([date, group]) => ({ date, tasks: sortDay(group) }));
}
