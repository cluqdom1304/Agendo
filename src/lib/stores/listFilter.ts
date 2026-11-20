// Filters of the «Tareas» view (F20-05). Only the list reads them: Hoy, the calendar and the
// inbox keep showing every task the tag filter lets through.
import { addDays } from "../features/calendar/grid";
import type { Task } from "../types";

export type DateFilter = "all" | "undated" | "overdue" | "today" | "week";

export interface ListFilter {
  date: DateFilter;
  /** Lowest priority shown: 0 shows every task (D-93). */
  minPriority: number;
  showCompleted: boolean;
}

export const NO_FILTER: ListFilter = {
  date: "all",
  minPriority: 0,
  showCompleted: true,
};

export function isFiltering(filter: ListFilter): boolean {
  return (
    filter.date !== "all" || filter.minPriority > 0 || !filter.showCompleted
  );
}

export function matchesFilter(
  task: Task,
  filter: ListFilter,
  today: string,
): boolean {
  if (!filter.showCompleted && task.isCompleted) return false;
  if (task.priority < filter.minPriority) return false;
  const due = task.dueDate;
  switch (filter.date) {
    case "all":
      return true;
    case "undated":
      return due === null;
    case "overdue":
      return due !== null && due < today && !task.isCompleted;
    case "today":
      return due === today;
    case "week":
      return due !== null && due >= today && due <= addDays(today, 6);
  }
}
