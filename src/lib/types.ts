// DTOs of the IPC contract (SPEC §7.2). Keep in sync with src-tauri/src/repo/model.rs; the
// fixture test in src/lib/types.test.ts fails if the Rust serialization drifts from these types.

export type EffectMode = "mica" | "acrylic" | "none";

export interface EffectApplied {
  requested: EffectMode;
  applied: EffectMode;
}

export type RepeatUnit = "hour" | "day" | "week" | "month";

/** How a custom-time reminder repeats (D-32). `count` and `until` are never both set. */
export interface Repeat {
  every: number;
  unit: RepeatUnit;
  /** Total number of soundings, first included. */
  count: number | null;
  /** Last local date, `YYYY-MM-DD`, inclusive. */
  until: string | null;
  /** Which sounding of the series this reminder is, starting at 1. */
  occurrence: number;
}

export interface Reminder {
  id: string;
  notifyAt: string;
  offsetMinutes: number | null;
  isSent: boolean;
  repeat: Repeat | null;
}

/** One entry of `reminders_set`: minutes before the task's date and time, or a UTC instant. */
export interface ReminderInput {
  offsetMinutes?: number;
  notifyAt?: string;
  /** Only together with `notifyAt`. */
  repeat?: Omit<Repeat, "occurrence"> & { occurrence?: number };
}

/** Payload of the `reminder:fired` event (SPEC §7.4). */
export interface ReminderFired {
  /** Set for the reminder of a task; `noteId` for the reminder of a note (D-52). */
  taskId?: string;
  noteId?: string;
  reminderId: string;
}

/** The time a note will remind its owner (F13-04). */
export interface NoteReminder {
  id: string;
  noteId: string;
  notifyAt: string;
}

export interface Task {
  id: string;
  title: string;
  contentJson: string;
  dueDate: string | null;
  dueTime: string | null;
  /** Minutes from `dueTime` to the end, within the same day; `null` = no duration (D-33). */
  durationMinutes: number | null;
  /** Note the task was created from (D-45); `null` if none or if that note is gone. */
  sourceNoteId: string | null;
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
  /** Live notes with the tag: tags are shared between tasks and notes (D-44). */
  noteCount: number;
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
  durationMinutes?: number;
  tagIds?: string[];
  contentJson?: string;
  sourceNoteId?: string;
}

/** Absent = leave as is; `null` = clear. */
export interface TaskPatch {
  title?: string;
  contentJson?: string;
  dueDate?: string | null;
  dueTime?: string | null;
  durationMinutes?: number | null;
}

export interface TagUpsertInput {
  id?: string;
  name: string;
  colorHex: string;
  parentId?: string;
}

/** Folder of notes (D-43). `parentId: null` is the top level. */
export interface Folder {
  id: string;
  name: string;
  parentId: string | null;
  position: number;
}

/** `noteCount` counts the live notes directly inside the folder, not in its subfolders. */
export interface FolderWithCount extends Folder {
  noteCount: number;
}

export interface FolderUpsertInput {
  id?: string;
  name: string;
  parentId?: string;
}

export interface Note {
  id: string;
  /** `null` is the inbox. */
  folderId: string | null;
  title: string;
  contentJson: string;
  isPinned: boolean;
  tagIds: string[];
  createdAt: string;
  updatedAt: string;
}

/** A note as listed: a short plain-text excerpt instead of the document. */
/** A saved version of a note, as listed (F13-03). */
export interface NoteVersionSummary {
  id: string;
  noteId: string;
  title: string;
  size: number;
  createdAt: string;
}

export interface NoteVersion {
  id: string;
  noteId: string;
  title: string;
  contentJson: string;
  createdAt: string;
}

/** A named document new notes can start from (F13-02). */
export interface NoteTemplate {
  id: string;
  name: string;
  contentJson: string;
  createdAt: string;
  updatedAt: string;
}

export interface NoteSummary {
  id: string;
  folderId: string | null;
  title: string;
  excerpt: string;
  isPinned: boolean;
  tagIds: string[];
  createdAt: string;
  updatedAt: string;
}

export type NotesScope = "all" | "inbox" | "pinned" | "folder";
export type NotesSort = "updated" | "created" | "title";

export interface NotesListQuery {
  scope: NotesScope;
  /** Required with `scope: "folder"`; includes the subfolders. */
  folderId?: string;
  tagId?: string;
  sort?: NotesSort;
}

export interface NoteCreateInput {
  folderId?: string;
  title?: string;
  contentJson?: string;
  tagIds?: string[];
}

/** Absent = leave as is. */
export interface NotePatch {
  title?: string;
  contentJson?: string;
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

/** A result of the unified search (F11-03): a task, a note or a tag. */
export type FoundItem =
  | {
      kind: "task";
      taskId: string;
      title: string;
      snippet: string;
      dueDate: string | null;
      isCompleted: boolean;
    }
  | {
      kind: "note";
      noteId: string;
      title: string;
      snippet: string;
      folderId: string | null;
      isPinned: boolean;
    }
  | {
      kind: "tag";
      tagId: string;
      name: string;
      colorHex: string;
      noteCount: number;
      pendingCount: number;
    };

/** A stored image of a note (D-47); `id` is the SHA-256 of its content. */
export interface Attachment {
  id: string;
  mime: string;
  sizeBytes: number;
}

/** Settings the backend accepts (`settings_get` / `settings_set`). */
export type SettingKey =
  | "view_mode"
  | "theme"
  | "window_effect"
  | "default_notify_time"
  | "app_section"
  | "notes_scope"
  | "notes_open"
  | "notes_recent";

/** Colour theme: follow the system or force one (F6-05). */
export type ThemeMode = "system" | "light" | "dark";

/** What the main area shows (F-10). */
export type ViewMode = "list" | "day" | "week" | "month";
