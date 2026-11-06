// Display order of tasks (D-05). Mirrors src-tauri/src/domain/ordering.rs so optimistic
// updates in the store land exactly where the backend would put them after a reload.
import type { Task } from "./types";

type Sortable = Pick<
  Task,
  "id" | "dueDate" | "dueTime" | "isCompleted" | "completedAt" | "position"
>;

/** Fixed-width, zero-padded formats compare correctly as text; `null` sorts first. */
function compareText(a: string | null, b: string | null): number {
  if (a === b) return 0;
  if (a === null) return -1;
  if (b === null) return 1;
  return a < b ? -1 : 1;
}

const compareId = (a: Sortable, b: Sortable) => compareText(a.id, b.id);

const byPosition = (a: Sortable, b: Sortable) =>
  a.position - b.position || compareId(a, b);

const byCompletion = (a: Sortable, b: Sortable) =>
  compareText(a.completedAt, b.completedAt) || compareId(a, b);

function byDay(a: Sortable, b: Sortable): number {
  if (a.isCompleted !== b.isCompleted) return a.isCompleted ? 1 : -1;
  if (a.isCompleted) return byCompletion(a, b);
  if (a.dueTime !== null && b.dueTime !== null) {
    return compareText(a.dueTime, b.dueTime) || byPosition(a, b);
  }
  if (a.dueTime !== null) return -1;
  if (b.dueTime !== null) return 1;
  return byPosition(a, b);
}

/** POR HACER: pending by position, then the completed ones. */
export function sortTodo<T extends Sortable>(tasks: readonly T[]): T[] {
  return [...tasks].sort((a, b) => {
    if (a.isCompleted !== b.isCompleted) return a.isCompleted ? 1 : -1;
    return a.isCompleted ? byCompletion(a, b) : byPosition(a, b);
  });
}

/** One day: timed, untimed, then completed. */
export function sortDay<T extends Sortable>(tasks: readonly T[]): T[] {
  return [...tasks].sort(byDay);
}

/** Dated tasks grouped by day, latest day first; undated tasks belong to no day. */
export function groupByDay<T extends Sortable>(
  tasks: readonly T[],
): { date: string; tasks: T[] }[] {
  const days = new Map<string, T[]>();
  for (const task of tasks) {
    if (task.dueDate === null) continue;
    const day = days.get(task.dueDate);
    if (day) day.push(task);
    else days.set(task.dueDate, [task]);
  }
  return [...days.entries()]
    .sort(([a], [b]) => compareText(b, a))
    .map(([date, group]) => ({ date, tasks: sortDay(group) }));
}

/** Splits an already sorted block into its pending part and its "Completadas" subsection. */
export function splitCompleted<T extends { isCompleted: boolean }>(
  sorted: readonly T[],
): { open: T[]; completed: T[] } {
  return {
    open: sorted.filter((task) => !task.isCompleted),
    completed: sorted.filter((task) => task.isCompleted),
  };
}
