import type { ReminderInput, Task } from "../types";

/** Where a new task goes: the "por hacer" list or one day of the agenda. */
export type DraftTarget =
  { section: "todo" } | { section: "agenda"; date: string };

/** An unsaved row at the end of a block, waiting for its title. */
export interface Draft {
  target: DraftTarget;
  /** Changes with every new draft so the input is rebuilt empty. */
  key: number;
}

export interface UndoEntry {
  key: number;
  task: Task;
}

export interface Notice {
  key: number;
  message: string;
  kind: "info" | "error";
}

export interface TaskBlock {
  /** `YYYY-MM-DD`, or `null` for POR HACER. */
  date: string | null;
  open: Task[];
  completed: Task[];
}

/** What `create` takes: the details of a new task. */
export interface NewTaskDetails {
  title: string;
  dueDate?: string;
  dueTime?: string;
  durationMinutes?: number;
  tagIds?: string[];
  contentJson?: string;
  reminders?: ReminderInput[];
  /** The note the task is created from (D-45). */
  sourceNoteId?: string;
  /** Repeat rule (F13-05); needs a date. */
  rrule?: string;
  /** 0 none … 3 urgent (D-93). */
  priority?: number;
}
