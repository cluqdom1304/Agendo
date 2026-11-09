// DTOs of the IPC contract (SPEC §7.2). Keep in sync with src-tauri/src/repo/model.rs; the
// fixture test in src/lib/types.test.ts fails if the Rust serialization drifts from these types.

export type EffectMode = "mica" | "acrylic" | "none";

export interface EffectApplied {
  requested: EffectMode;
  applied: EffectMode;
}

export interface Reminder {
  id: string;
  notifyAt: string;
  offsetMinutes: number | null;
  isSent: boolean;
}

/** One entry of `reminders_set`: minutes before the task's date and time, or a UTC instant. */
export interface ReminderInput {
  offsetMinutes?: number;
  notifyAt?: string;
}

/** Payload of the `reminder:fired` event (SPEC §7.4). */
export interface ReminderFired {
  taskId: string;
  reminderId: string;
}

export interface Task {
  id: string;
  title: string;
  contentJson: string;
  dueDate: string | null;
  dueTime: string | null;
  isCompleted: boolean;
  completedAt: string | null;
  position: number;
  tagIds: string[];
  reminders: Reminder[];
  createdAt: string;
  updatedAt: string;
}

export interface Tag {
  id: string;
  name: string;
  colorHex: string;
  parentId: string | null;
  position: number;
}

export interface TagWithCount extends Tag {
  pendingCount: number;
}

export interface DayGroup {
  date: string;
  tasks: Task[];
}

export type ListView = "todo" | "agenda" | "range";
export type Section = "todo" | "agenda";

export interface TasksSections {
  todo: Task[];
  agenda: DayGroup[];
}

export interface TasksListQuery {
  view: ListView;
  from?: string;
  to?: string;
  tagId?: string;
}

export interface TaskCreateInput {
  title?: string;
  dueDate?: string;
  dueTime?: string;
  tagIds?: string[];
  contentJson?: string;
}

/** Absent = leave as is; `null` = clear. */
export interface TaskPatch {
  title?: string;
  contentJson?: string;
  dueDate?: string | null;
  dueTime?: string | null;
}

export interface TagUpsertInput {
  id?: string;
  name: string;
  colorHex: string;
  parentId?: string;
}

export type ErrorCode =
  "NOT_FOUND" | "VALIDATION" | "CONFLICT" | "DB" | "IO" | "NOTIFY" | "INTERNAL";

export interface SearchHit {
  taskId: string;
  title: string;
  /** Fragment of the notes around the match; empty when the task has no notes. */
  snippet: string;
  dueDate: string | null;
  isCompleted: boolean;
}

/** Settings the backend accepts (`settings_get` / `settings_set`). */
export type SettingKey = "view_mode";

/** What the main area shows (F-10). */
export type ViewMode = "list" | "day" | "week" | "month";
